#include <stdio.h>
#include <windows.h>
#include <xinput.h>

#define eprintf(...) fprintf(stderr, ##__VA_ARGS__)
#define ret_if_not(condition, code) \
  if (!(condition)) return code

typedef struct {
  HMODULE(__cdecl *GetModuleHandleA)(LPCSTR);
  FARPROC(__stdcall *GetProcAddress)(HMODULE, LPCSTR);
  BOOL(__stdcall *VirtualProtect)(LPVOID, SIZE_T, DWORD, PDWORD);
  char strDll[14];
  char strFunc[15];
  char strCheck[25];
  char strReplace[25];
  size_t patchOffset;
} ParamsEx;

DWORD inject_code(DWORD pid, LPCVOID fun, SIZE_T funSize, LPCVOID params, SIZE_T paramsSize) {
  DWORD ExitCode = 1;

  HANDLE hProcess = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
  if (!hProcess) {
    eprintf("OpenProcess error\n");
    goto open_error;
  }

  LPVOID paramsAddr = VirtualAllocEx(hProcess, NULL, funSize + paramsSize, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
  if (!paramsAddr) {
    eprintf("VirtualAllocEx error\n");
    goto alloc_error;
  }

  LPVOID funAddr = (BYTE *)paramsAddr + paramsSize;
  if (!WriteProcessMemory(hProcess, paramsAddr, params, paramsSize, NULL) ||
      !WriteProcessMemory(hProcess, funAddr, fun, funSize, NULL)) {
    eprintf("WriteProcessMemory error\n");
    goto write_error;
  }

  HANDLE hThread = CreateRemoteThread(hProcess, NULL, 0, funAddr, paramsAddr, 0, NULL);
  if (!hThread) {
    eprintf("CreateRemoteThread error\n");
    goto thread_error;
  }

  if (WaitForSingleObject(hThread, INFINITE) != WAIT_OBJECT_0 || !GetExitCodeThread(hThread, &ExitCode)) {
    eprintf("Function error\n");
  };

  CloseHandle(hThread);
thread_error:
write_error:
  VirtualFreeEx(hProcess, paramsAddr, funSize + paramsSize, MEM_RELEASE);
alloc_error:
  CloseHandle(hProcess);
open_error:
  return ExitCode;
}

DWORD patch_fun(const ParamsEx *params) {
  HMODULE xlib = params->GetModuleHandleA(params->strDll);
  ret_if_not(xlib, 2);
  FARPROC func = params->GetProcAddress(xlib, params->strFunc);
  ret_if_not(func, 3);
  char *patchAddr = (size_t)func + params->patchOffset;

  size_t pos = 0;
  while (pos < sizeof(params->strCheck) && patchAddr[pos] == params->strCheck[pos]) ++pos;
  ret_if_not(pos == sizeof(params->strCheck), 4);

  DWORD old;
  params->VirtualProtect(patchAddr, sizeof(params->strReplace), PAGE_READWRITE, &old);

  pos = sizeof(params->strReplace);
  while (pos-- > 0) (patchAddr)[pos] = params->strReplace[pos];

  params->VirtualProtect(patchAddr, sizeof(params->strReplace), old, &old);

  return 0;
}

int main(int argc, char const *argv[]) {
  if (argc != 2) {
    eprintf("argc != 2\n");
    return 1;
  }
  int pid = atoi(argv[1]);

  int fun_len = 0;
  while (((BYTE *)patch_fun)[fun_len++] != 0xC3);
  ParamsEx params = {
      GetModuleHandleA,
      GetProcAddress,
      VirtualProtect,
      "XInput1_4.dll",
      "XInputGetState",
      "\x8b\xc3\x48\x8b\x5c\x24\x50\x48\x83\xc4\x30\x41\x5e\x5f\x5e\xc3\xcc\xcc\xcc\xcc\xcc\xcc\xcc\xcc\xcc",
      "\xb8\xff\x00\x00\x00\x66\x31\x47\x07\x8b\xc3\x48\x8b\x5c\x24\x50\x48\x83\xc4\x30\x41\x5e\x5f\x5e\xc3",
      340,
  };

  eprintf("fun_len = %d\n", fun_len);

  int exit_code = inject_code(pid, &patch_fun, fun_len, &params, sizeof(params));
  printf("exit_code = %u\n", exit_code);

  return 0;
}

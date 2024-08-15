#include <stdio.h>
#include <Windows.h>
#include <Xinput.h>

#define eprintf(...) fprintf(stderr, ##__VA_ARGS__)

typedef struct {
  HMODULE(__cdecl* GetModuleHandleA)(LPCSTR);
  FARPROC(__stdcall* GetProcAddress)(HMODULE hModule, LPCSTR lpProcName);
  int(__stdcall* MessageBoxA)(HWND hWnd, LPCSTR lpText, LPCSTR lpCaption, UINT uType);  // test
  char strDll[14];
  char strFunc[15];
} ParamsEx;

int inject_code(DWORD pid, LPCVOID fun, SIZE_T funSize, LPCVOID params, SIZE_T paramsSize) {
  int status = 1;

  HANDLE hProcess = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
  if (!hProcess) {
    eprintf("OpenProcess error");
    goto open_error;
  }

  LPVOID paramsAddr = VirtualAllocEx(hProcess, NULL, funSize + paramsSize, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
  if (!paramsAddr) {
    eprintf("VirtualAllocEx error");
    goto alloc_error;
  }

  LPVOID funAddr = (BYTE*)paramsAddr + paramsSize;
  if (!WriteProcessMemory(hProcess, paramsAddr, params, paramsSize, NULL) ||
      !WriteProcessMemory(hProcess, funAddr, fun, funSize, NULL)) {
    eprintf("WriteProcessMemory error");
    goto write_error;
  }

  HANDLE hThread = CreateRemoteThread(hProcess, NULL, 0, funAddr, paramsAddr, 0, NULL);
  if (!hThread) {
    eprintf("CreateRemoteThread error");
    goto thread_error;
  }

  WaitForSingleObject(hThread, INFINITE);

  status = 0;
  CloseHandle(hThread);
thread_error:
write_error:
  VirtualFreeEx(hProcess, paramsAddr, funSize + paramsSize, MEM_RELEASE);
alloc_error:
  CloseHandle(hProcess);
open_error:
  return status;
}

DWORD test_func(ParamsEx* params) {
  params->MessageBoxA(0, params->strDll, params->strFunc, 0);
  return 0;
}

int main(int argc, char const* argv[]) {
  if (argc != 2) {
    eprintf("argc != 2\n");
    return 1;
  }
  int pid = atoi(argv[1]);

  int fun_len = 0;
  while (((BYTE*)test_func)[fun_len++] != 0xC3);
  ParamsEx params = {GetModuleHandleA, GetProcAddress, MessageBoxA, "XInput1_4.dll", "XInputGetState"};

  int status = inject_code(pid, &test_func, fun_len, &params, sizeof(params));
  if (status) {
    printf("failed");
  } else {
    printf("zaebis");
  }

  return status;
}

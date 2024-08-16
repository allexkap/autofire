#include <stdio.h>
#include <Windows.h>
#include <Xinput.h>

#define eprintf(...) fprintf(stderr, ##__VA_ARGS__)
#define ret_if_not(condition, code) \
  if (!condition) return code

typedef struct {
  HMODULE(__cdecl *GetModuleHandleA)(LPCSTR);
  FARPROC(__stdcall *GetProcAddress)(HMODULE, LPCSTR);
  BOOL(__stdcall *VirtualProtect)(LPVOID, SIZE_T, DWORD, PDWORD);
  char strDll[14];
  char strFunc[15];
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

DWORD test_func(ParamsEx *params) {
  HMODULE xlib = params->GetModuleHandleA(params->strDll);
  ret_if_not(xlib, 3);
  FARPROC func = params->GetProcAddress(xlib, params->strFunc);
  ret_if_not(func, 4);

  size_t imageBase = (size_t)params->GetModuleHandleA(NULL);
  ret_if_not(imageBase, 5);

  PIMAGE_DOS_HEADER dosHeader = (PIMAGE_DOS_HEADER)(imageBase);
  PIMAGE_NT_HEADERS ntHeaders = (PIMAGE_NT_HEADERS)(imageBase + dosHeader->e_lfanew);
  PIMAGE_OPTIONAL_HEADER optionalHeader = &(ntHeaders->OptionalHeader);  // o_o

  IMAGE_DATA_DIRECTORY importDirectory = optionalHeader->DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
  PIMAGE_IMPORT_DESCRIPTOR importDescriptor = (PIMAGE_IMPORT_DESCRIPTOR)(imageBase + importDirectory.VirtualAddress);

  while (importDescriptor->Name) {
    PIMAGE_THUNK_DATA thunk = (PIMAGE_THUNK_DATA)(imageBase + importDescriptor->FirstThunk);
    while (thunk->u1.AddressOfData) {
      if (thunk->u1.Function == (size_t)func) {
        void *ptr = &thunk->u1.Function;
        DWORD old;
        params->VirtualProtect(ptr, 8, PAGE_READWRITE, &old);
        thunk->u1.Function = (size_t)thunk->u1.Function;  // test
        params->VirtualProtect(ptr, 8, old, &old);
        return 0;
      }
      ++thunk;
    }
    ++importDescriptor;
  }
  return 2;
}

int main(int argc, char const *argv[]) {
  if (argc != 2) {
    eprintf("argc != 2\n");
    return 1;
  }
  int pid = atoi(argv[1]);

  int fun_len = 0;
  while (((BYTE *)test_func)[fun_len++] != 0xC3);
  ParamsEx params = {GetModuleHandleA, GetProcAddress, VirtualProtect, "XInput1_4.dll", "XInputGetState"};

  eprintf("fun_len = %d\n", fun_len);

  int exit_code = inject_code(pid, &test_func, fun_len, &params, sizeof(params));
  printf("exit_code = %u\n", exit_code);

  return 0;
}

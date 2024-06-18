from pathlib import Path
from shutil import copy2

bytecode = (
    b'\x8B\xC3\x48\x8B\x5C\x24\x50\x48\x83\xC4\x30\x41\x5E\x5F\x5E\xC3\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC\xCC',
    b'\xB8\xFF\x00\x00\x00\x66\x31\x47\x07\x8B\xC3\x48\x8B\x5C\x24\x50\x48\x83\xC4\x30\x41\x5E\x5F\x5E\xC3',
)
dll_path = Path('C:/Windows/System32/XInput1_4.dll')
default_game_path = Path('C:/Program Files (x86)/Steam/steamapps/common/Cuphead')

print(f'default path {default_game_path}')
game_path = Path(input('custom path or nothing: '))
game_dll_path = game_path / dll_path.name
copy2(dll_path, game_dll_path)

with open(game_dll_path, 'rb+') as file:
    data = file.read()
    n = data.count(bytecode[0])
    if n < 1:
        print('not found')
    elif n > 1:
        print('multiple matches')
    else:
        pos = data.find(bytecode[0])
        file.seek(pos)
        file.write(bytecode[1])
        print('successfully')

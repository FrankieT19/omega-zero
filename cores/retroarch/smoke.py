#!/usr/bin/env python3
"""Exercise the packaged ARM64 RetroArch through the exact private stdin bridge."""
import pathlib, queue, subprocess, sys, tempfile, threading, time
exe, core = map(lambda p: str(pathlib.Path(p).resolve()), sys.argv[1:3])
with tempfile.TemporaryDirectory(prefix='omega-ra-') as temp:
    root=pathlib.Path(temp); rom=root/'probe.bin'; rom.write_bytes(b'probe')
    cfg=root/'probe.cfg'
    cfg.write_text(f'''video_driver = "null"
audio_driver = "null"
input_driver = "omega"
input_joypad_driver = "omega"
stdin_cmd_enable = "true"
network_cmd_enable = "false"
menu_driver = "rgui"
config_save_on_exit = "false"
confirm_quit = "false"
savestate_file_compression = "false"
savestate_auto_save = "true"
sort_savefiles_enable = "false"
sort_savestates_enable = "false"
rewind_enable = "true"
input_player1_a_btn = "8"
input_player1_up_btn = "4"
savefile_directory = "{root}"
savestate_directory = "{root}"
''')
    with (root/'log').open('w') as log:
        p=subprocess.Popen(['/lib/ld-linux-aarch64.so.1',exe,'--config',str(cfg),'-L',core,str(rom),'--verbose'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=log,text=True,bufsize=1)
        replies=queue.Queue(); seen=[]
        def reader():
            for line in p.stdout: replies.put(line.strip())
        threading.Thread(target=reader,daemon=True).start()
        def send(text): p.stdin.write(text+'\n'); p.stdin.flush()
        def expect(command, pattern, timeout=15):
            end=time.monotonic()+timeout
            while time.monotonic()<end:
                if p.poll() is not None: raise AssertionError('RetroArch exited '+str(p.returncode))
                send(command)
                try:
                    while True:
                        line=replies.get(timeout=.15);seen.append(line)
                        if pattern in line:return line
                except queue.Empty: pass
            raise AssertionError(f'No {pattern!r}; replies={seen[-20:]}')
        try:
            expect('GET_STATUS','GET_STATUS PLAYING')
            send('OMEGA_INPUT 110 0') # libretro A + Up, not a device-specific evdev code.
            time.sleep(.2)
            send('SAVE_STATE_SLOT 0')
            end=time.monotonic()+5
            while not (root/'probe.state').exists() and time.monotonic()<end: time.sleep(.05)
            assert (root/'probe.state').stat().st_size>32
            send('OMEGA_INPUT 0 1')
            expect('GET_OPTION fast_forward','fast_forward\ton')
            send('OMEGA_INPUT 0 0')
            expect('GET_OPTION fast_forward','fast_forward\toff')
            expect('SET_OPTION menu on\nGET_OPTION menu','menu\ton')
            expect('SET_OPTION menu off\nGET_OPTION menu','menu\toff')
            send('LOAD_STATE_SLOT 0\nOMEGA_SCREEN\nOMEGA_PARAM OMEGA_DIM 0.25')
            expect('SET_OPTION pause on\nGET_STATUS','GET_STATUS PAUSED')
            expect('SET_OPTION pause off\nGET_STATUS','GET_STATUS PLAYING')
            send('OMEGA_QUIT'); assert p.wait(timeout=10)==0
            battery=(root/'probe.srm').read_bytes()
            assert battery[:2]==b'\x01\x01',f'Gamepad bridge failed: {battery[:8]!r}'
            assert (root/'probe.state.auto').stat().st_size>32
            print('RetroArch native smoke passed: real core input, save/load, fast-forward hold/release, menu, pause/resume, clean exit and battery/autostate persistence')
        except BaseException:
            if p.poll() is None: p.kill();p.wait()
            print((root/'log').read_text()[-12000:],file=sys.stderr)
            raise

# vlc-http-sync
Syncronize vlc players using the vlc http interface.
----

Usage:
- Load the same file into 2 or more VLC media players.
- Select one of them as the main player. Other players will syncronize to it.
- Enable 'http interface' with password '1234' on port '8080' on all the vlc players.
- Forward port 8080 of the machine running main vlc player.
- Run vlc-sync.exe on all other (excluding main) machines passing the ip of main machine as commandline argument.
    - eg. vlc-sync.exe "http://ip-of-main-machine:8080"

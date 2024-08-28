import os

os.system("cargo build --target x86_64-pc-windows-gnu")
os.system("rm /mnt/c/Users/Konra/Desktop/ESP-Stomp-Controller.exe")
os.system("cp target/x86_64-pc-windows-gnu/debug/ESP-Stomp-Controller.exe /mnt/c/Users/Konra/Desktop/ESP-Stomp-Controller.exe")
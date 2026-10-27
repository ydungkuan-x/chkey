regsvr32 "C:\Users\ydung\Desktop\Keyboard\CHKey\target\debug\chkey_tsf.dll"

regsvr32 /u "C:\Users\ydung\Desktop\Keyboard\CHKey\target\debug\chkey_tsf.dll"

/s

cargo build --manifest-path C:\Users\ydung\Desktop\Keyboard\CHKey\platforms\windows\Cargo.toml

cargo build --lib
cargo build --lib 2>&1

taskkill /F /IM ctfmon.exe
ctfmon.exe

ox
aa ox aa oo ax ax o ox o oxx ox i a aa a i b o

# 1. Build DLL TSF (nhúng sẵn rade.ico)

cargo build -p chkey_tsf --release

# 2. Build ứng dụng chính CHKey.exe

cargo build -p chkey_windows --release

# 3. Build bộ cài đặt (sẽ tự động đóng gói cả 2 file trên vào bên trong)

cargo build -p chkey_setup --bin CHKey_Setup --release

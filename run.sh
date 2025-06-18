cross clean 

# cross build --target=arm-unknown-linux-gnueabihf --release
cross build --target=arm-unknown-linux-musleabi

scp target/arm-unknown-linux-musleabi/debug/poc lyrocs@rusty.local:~/
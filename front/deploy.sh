trunk build --release
rsync -av --delete ./dist/  lyrocs@rusty.local:~/dist
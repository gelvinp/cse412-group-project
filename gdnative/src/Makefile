.PHONY: win64
win64:
	cargo build --release
	powershell -Command "Copy-Item ./target/release/lib412.dll ../lib412/win-x64/lib412.dll"
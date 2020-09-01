cargo build --release
copy target\release\loguinho.exe choco\tools\loguinho.exe
pushd choco
choco pack
popd
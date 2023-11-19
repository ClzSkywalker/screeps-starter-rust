rustup toolchain install nightly
rustup component add rust-src --toolchain nightly-x86_64-pc-windows-msvc // win
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu // ubuntu
rustup override set nightly 将当前目录设置为nightly版本的编译与开发
rustup override unset 取消当前目录设置为nightly版本的编译与开发

cargo +nightly screeps deploy


Game.spawns['Spawn1'].spawnCreep([WORK, CARRY, MOVE], 'Worker1');

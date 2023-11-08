rustup toolchain install nightly
rustup component add rust-src --toolchain nightly-x86_64-pc-windows-msvc
rustup override set nightly 将当前目录设置为nightly版本的编译与开发
rustup override unset 取消当前目录设置为nightly版本的编译与开发



Game.spawns['Spawn1'].spawnCreep([WORK, CARRY, MOVE], 'Worker1');
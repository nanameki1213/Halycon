# Halycon

Halycon はRISC-V 64上で動作するハイパーバイザです。

セキュリティ・キャンプ２０２４で作成したAArch64向けのハイパーバイザをRISC-Vに移植しています。

## 環境
* QEMU(>= 9.0.2)

## ビルド方法
```
rustup target add riscv64gc-unknown-none-elf

git clone https://github.com/nanameki1213/Halycon.git

# run hypervisor
cd Halycon
cargo run --release
```

## TODO
- [x] 2段階ページング
- [x] ゲストへの移行
- [ ] シリアルデバイスの仮想化
- [ ] virtio-blkデバイスの実装
- [ ] ネットワークの仮想化
- [ ] 複数ゲストの起動
- [ ] Linuxの起動

## 記事
ハイパーバイザを開発する際にRISC-Vの仕様を整理するために書いたページです。
http://shinysheep.net/riscv-hypervisor/

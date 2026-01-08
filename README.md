# Halycon

Halycon はRISC-V 64上で動作するハイパーバイザです。

セキュリティ・キャンプ２０２４で作成したAArch64向けのハイパーバイザをRISC-Vに移植しています。

現在はゲストとしてu-bootが動作します。

## 環境
* QEMU(>= 9.0.2)

## ビルド方法
### U-Bootのブートスクリプトのコンパイル
```
cd tools
./boot.sh
```
### ゲスト用のu-bootのビルド
```
cd u-boot
# ビルドに失敗する場合は、riscv64-linux-gnu-gcc, make, gcc, bc, bison, flex, libssl-dev, python3, pkg-config, device-tree-compiler などのビルド用パッケージがインストールされているか確認してください
./build.sh
```
u-boot/u-boot.binを、Halycon/bin/配下にコピーする

### Halyconのビルド
```
rustup target add riscv64gc-unknown-none-elf

# run hypervisor
cargo xtask build -f nested && cargo xtask run
```

## Linuxをミニマムで動かすロードマップ
- [x] 2段階ページング
- [x] ゲストへの移行
- [x] ゲスト用u-bootのロード
    - [x] virtioからのファイル読み出し
    - [x] ファイルのメモリ上への展開
    - [x] ゲスト用u-bootへ渡すデバイスツリーの用意
- [x] 仮想マシンを起動するために必要な情報をまとめた構造体の用意
- [ ] RISC-V AIA (Advanced Interrupt Architecture)への対応
    - [ ] 割り込みコントローラ(aplic)のデバイスドライバの実装
    - [ ] IMSICのドライバの実装
- [ ] デバイス仮想化
    - [x] シリアルデバイスの仮想化
    - [x] SBIコールの仮想化
    - [x] タイマデバイスの仮想化
    - [x] ブロックデバイスの仮想化
        - [x] virtio-blkデバイスの実装(パススルー)
    - [ ] 割り込みコントローラの仮想化
- [ ] Linuxの起動

## TODO
- [x] メモリアロケータの実装
    - [x] デバイスツリーの解析によるデバイスの把握
- [x] virtio queueを使いまわさずにRing構造を利用する
- [ ] virtio-blkの読み書き完了を割り込みによって把握する
- [x] デバイスツリーの解析によるデバイスの把握
- [ ] ネットワークの仮想化
- [ ] 複数ゲストの起動
- [ ] ファイルシステムの実装
- [ ] UARTの完全仮想化

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

## Linuxをミニマムで動かすロードマップ
- [x] 2段階ページング
- [x] ゲストへの移行
- [x] ゲスト用u-bootのロード
    - [x] virtioからのファイル読み出し
    - [x] ファイルのメモリ上への展開
    - [x] ゲスト用u-bootへ渡すデバイスツリーの用意
- [x] 仮想マシンを起動するために必要な情報をまとめた構造体の用意
- [ ] デバイス仮想化
    - [ ] シリアルデバイスの仮想化
    - [ ] 例外の仮想化
    - [ ] タイマデバイスの仮想化
    - [ ] ブロックデバイスの仮想化
        - [ ] virtio-blkデバイスの実装
    - [ ] 割り込みコントローラの仮想化
- [ ] Linuxの起動

## TODO
- [ ] メモリアロケータの実装
- [x] virtio queueを使いまわさずにRing構造を利用する
- [ ] virtio-blkの読み書き完了を割り込みによって把握する
- [ ] デバイスツリーの解析によるデバイスの把握
- [ ] ネットワークの仮想化
- [ ] 複数ゲストの起動
- [ ] ファイルシステムの実装
- [ ] 割り込み時のコンテキストをスタック上ではなくVM構造体に直接保存

## 記事
ハイパーバイザを開発する際にRISC-Vの仕様を整理するために書いたページです。
<p>http://shinysheep.net/riscv-hypervisor/</p>

# rCore-fat
Develop fat32 file system for rcore-Tutorial-v3


## Usage
```shell
cd os
make run
```

## Summary
[FAT32 File System Summary](https://github.com/KuangjuX/FAT32/blob/main/README.md)   
   
### 内核中的文件系统
在内核中，我们使用 OSInode 来表示文件，该结构体包含读写标签、当前偏移、以及对应虚拟文件的引用。对于文件和目录，在内核中都使用 OSInode 来描述，而对于其他可读写对象，例如设备、Pipe则被当作抽象文件处理。

在 rCore-Tutorial 的文件系统中使用 File Trait 来描述抽象文件，并为每种文件类型实现 Trait 中对应的方法，当系统调用操作文件时，则调用 Trait 中对应的方法来操作文件，但对于 FAT32 文件系统来说，对于操作 FAT32 的实际文件来说是不够的，因此我在这里将文件分为了两类，一类为真实的文件，一类则为抽象文件（复用 File Trait）用来描述 Stdio、网卡等设备抽象文件的调用。

### 文件系统库
在 FAT32 文件系统库的设计中，我们使用 `fat_manager` 来统一管理 FAT32 文件系统的磁盘内容：   
   
在操作系统启动时， `fat32_manager` 首先启动文件系统，引导扇区的数据并进行校验。`fat_manager` 首先会读入 0 号扇区，获得隐藏扇区数并初始化缓存偏移量，之后读取逻辑 0 扇区，即引导扇区，获取 FAT32 的基本信息，随后读取u FSInfo 扇区，获取簇信息，进行签名校验。
   
当获取文件系统的元信息之后，`fat_manager` 会根据已有信息计算 FAT 所处的位置，初始化 `FAT` 结构体，然后根据已有信息生成虚拟根目录项，随后返回 `fat_manager` 供操作系统调用。

# Warpai Linux 版本

[English](LINUX.md) | **简体中文**

Linux 桌面发布工程覆盖 x86_64、ARM64，与
[Warp 官方安装范围](https://docs.warp.dev/getting-started/quickstart/installation-and-setup)
一致。兼容性目标为 glibc 2.31 及以上、OpenGL ES 3.0+ 或 Vulkan，涵盖
Ubuntu 20.04+、Debian 11+、Fedora 32+、Arch，以及 OpenSUSE/SLE 等兼容 RPM
系统。不覆盖旧版 glibc 或基于 musl 的系统。

当前 Linux 包是验证产物，尚未作为正式支持的桌面版本发布。原生 Wayland、
桌面环境、凭据和电源管理的剩余验收见[Linux 技术约定](../specs/linux-desktop/TECH.md)。
能够生成某种格式的安装包，不代表已经在每个发行版上完成验收。

## 安装格式

两种架构均生成以下四种格式；文件名中的 `ARCH` 为 `x64` 或 `arm64`。

| 发行版 | 安装包 | 安装命令 |
| --- | --- | --- |
| Debian / Ubuntu | `Warpai-VERSION-linux-ARCH.deb` | `sudo apt install ./Warpai-VERSION-linux-ARCH.deb` |
| Fedora / 兼容 RPM 系统 | `Warpai-VERSION-linux-ARCH.rpm` | `sudo dnf install ./Warpai-VERSION-linux-ARCH.rpm` |
| OpenSUSE / SLE | 同一 RPM 包 | `sudo zypper install ./Warpai-VERSION-linux-ARCH.rpm` |
| Arch | `Warpai-VERSION-linux-ARCH.pkg.tar.zst` | `sudo pacman -U ./Warpai-VERSION-linux-ARCH.pkg.tar.zst` |
| 其他兼容桌面 | `Warpai-VERSION-linux-ARCH.AppImage` | 添加执行权限后运行 |

原生安装包提供 `/usr/bin/warpai`、桌面入口和图标；主程序、Agent 通信桥及资源
位于 `/opt/warpai`。安装过程不添加 APT/YUM/pacman 软件源或上游签名密钥。
本地 Worktree 和 Review 需要系统 Git；各 CLI Agent 的依赖和认证仍由用户管理。
精简桌面还需具备图形驱动、字体、`xdg-utils`、文件对话框支持和桌面 D-Bus 服务。

切换语言后会等待旧进程退出，再启动新实例。AppImage 会重新执行原始安装包，
因此该文件必须保留在原路径。更新检查会选择对应架构已发布的 AppImage；原生包
用户可在同一 Release 页面选择所需格式。

## 构建与交付

手动触发 **Release Warpai Linux**，填写语义版本 `release_tag`。默认仅生成测试
产物，使用选定的源码版本。两种架构均在 Debian 11 环境构建，并检查 ELF 的实际
glibc 要求、架构、版本、通信桥、资源和桌面标识。工作流执行语言重启回归、
默认和 `warp_platform` 编译检查，以及安装后的 X11 桌面启动截图，输出八个包、
源码清单和 SHA-256。通信验证工作流中的可选 Linux 检查复用同一实现。

若要附加到 Release，先准备经过审阅的不可变 tag 和已有草稿，再关闭仅产物模式。
工作流要求两种架构全部通过，并核对源码和校验和；拒绝覆盖现有附件，拒绝向已
发布版本附加文件，不会自动发布 Release。仅产物模式不修改 GitHub Releases。

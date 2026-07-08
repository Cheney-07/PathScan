# PathScan
# 使用文档
## 安装
首先检查是否拥有rust环境
```bash
rustc --version
```
下载并编译
```bash
git clone https://github.com/Cheney-07/PathScan.git
cd PathScan
cargo build --release

```
## 基本用法

```bash
pathscan -u <目标URL> [选项]
```

# 全部参数说明

## 目标

| 参数 | 说明 |
| --- | --- |
| `-u, --url <URL>` | 必填，目标扫描 URL |


## 字典

| 参数 | 说明 |
| --- | --- |
| `-w, --wordlist <PATH>` | 外部字典文件路径 |
| `--builtin <SIZE>` | 使用内置字典，可选 `small` / `medium` / `large`（默认 `medium`） |

## 请求配置

| 参数 | 说明 |
| --- | --- |
| `--concurrency <N>` | 并发请求数（默认 32） |
| `--rate-limit <N>` | 每秒请求上限（默认无限制） |
| `--timeout <SECS>` | 单个请求超时秒数（默认 10） |
| `-H, --header <KEY:VALUE>` | 自定义请求头，可重复使用 |
| `--cookie <STR>` | Cookie 字符串 |
| `--user-agent <STR>` | 自定义 User-Agent |
| `--proxy <URL>` | 代理地址（如 `http://127.0.0.1:8080`） |

## 过滤器

| 参数 | 说明 |
| --- | --- |
| `--status <CODES>` | 筛选目标状态码，逗号分隔（如 `200,301,403`） |
| `--no-length-filter` | 关闭响应长度过滤（默认开启，过滤与基线长度相同的结果） |
| `--content-regex <REGEX>` | 按内容正则匹配过滤结果 |

## 递归扫描

| 参数 | 说明 |
| --- | --- |
| `-r, --recursive` | 开启递归扫描，发现目录时自动深入 |
| `--depth <N>` | 最大递归深度（默认 1） |

## 输出

| 参数 | 说明 |
| --- | --- |
| `-o, --output <PATH>` | 输出文件前缀（默认 `pathscan_report`） |
| `--json` | 导出 JSON 格式报告 |
| `--html` | 导出 HTML 格式报告 |
| `--no-color` | 关闭终端颜色输出 |

## 其他

| 参数 | 说明 |
| --- | --- |
| `--no-baseline` | 跳过基线校准（软 404 检测） |
| `--tui` | 启动 TUI 交互模式 |


# 示例

## 使用内置小字典扫描

```bash
pathscan -u https://example.com --builtin small
```


## 使用外部字典 + 自定义并发 + JSON/HTML 报告

```bash
pathscan \
-u https://example.com \
-w ./my_wordlist.txt \
--concurrency 64 \
--json \
--html \
-o scan_result
```


## 递归扫描 + 自定义请求头 + 代理

```bash
pathscan \
-u https://example.com \
--builtin medium \
-r \
--depth 3 \
-H "Authorization: Bearer xxx" \
--proxy http://127.0.0.1:8080
```


## 仅关注 200 和 403 + 关闭长度过滤

```bash
pathscan \
-u https://example.com \
--builtin large \
--status 200,403 \
--no-length-filter
```


## TUI 交互模式

```bash
pathscan \
-u https://example.com \
--builtin medium \
--tui
```


# TUI 快捷键

| 按键 | 功能 |
| --- | --- |
| `p` | 暂停 / 继续扫描 |
| `q` | 退出 |
| `s` | 保存当前结果到文件 |
| `f` | 打开过滤器 |
| `Tab` | 切换焦点区域（日志 / 统计） |
| `↑ ↓` | 浏览结果 |
| `r` | 切换递归模式 |


# 输出文件

扫描完成后生成：
## JSON 报告

```text
xxx.json
```
包含完整结构化结果：
- URL
- 状态码
- Content-Type
- 响应长度
- 响应时间
- 重定向位置
- 
## HTML 报告

```text
xxx.html
```
生成可搜索的静态报告页面：
- 扫描摘要统计
- 结果列表
- 搜索功能
- 状态码统计

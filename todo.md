# Shirone CLI TODO

## 第一阶段：模板文章创建（已完成）

- [x] 读取 `config.toml` 并支持命令行覆盖
- [x] 使用用户模板创建 `<slug>/index.md`
- [x] 支持多级 slug、路径安全检查、`--dry-run`、`--force`
- [x] 生成 Shirone 兼容的日期字段和 frontmatter 变量
- [x] 从图片接口下载并转换为 `<slug>/cover.webp`
- [x] 使用临时文件和同步写入避免半成品文件
- [x] 编辑器支持 enabled、open_file、wait、非交互跳过、失败警告、路径和 `{file}` 解析
- [x] 为后续命令保留 CLI 入口

## 后续阶段

- [ ] `update`：只更新 frontmatter，保留正文和未知字段
- [ ] `content status`：检查内容源、挂载和同步状态
- [ ] `content sync`：实现本地内容仓增量物化和 `keep/prune`
- [ ] `content watch`：监听本地内容仓并触发增量同步
- [ ] `content clean`：清理物化内容并支持备份还原
- [ ] `content export`：将代码仓内容安全写回内容仓
- [ ] `content eject`：单仓迁移到双仓结构
- [ ] `validate` / `content validate`：实现 frontmatter、资源和配置检查
- [ ] `publish`：实现 Git status、commit、push 接口
- [ ] 支持 JSON 图片接口、图片缓存和可配置 WebP 编码质量
- [ ] 支持 moments、series、spec 等其他 Shirone 内容类型

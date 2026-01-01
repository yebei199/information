# Spider 高级 API 指南 (Web Automation)

`spider` 库通过 `WebAutomation` 提供了一套高级的浏览器自动化 API，允许在爬虫执行过程中进行复杂的页面交互（CDP 自动化），而无需直接操作低层的 `chromiumoxide`。

## 1. 核心概念：WebAutomation

`WebAutomation` 是一个枚举类型，定义了可以在页面上执行的各种操作。你可以通过 `HashMap<String, Vec<WebAutomation>>` 将特定的操作序列绑定到匹配的 URL 路径上。

### 常用操作变体：

- **`Evaluate(String)`**: 执行自定义 JavaScript 代码。
- **`Wait(u64)`**: 等待指定的毫秒数。
- **`Click(String)`**: 根据 CSS 选择器点击元素。
- **`Fill { selector: String, value: String }`**: 向输入框填充内容。
- **`Type { value: String, modifier: Option<i64> }`**: 模拟键盘输入。
- **`ScrollY(i32)`**: 垂直滚动页面。
- **`WaitFor(String)`**: 等待指定的 CSS 选择器出现。
- **`WaitForAndClick(String)`**: 等待元素出现并点击。
- **`Screenshot { ... }`**: 截取页面截图。
- **`InfiniteScroll(u32)`**: 执行无限滚动。

## 2. 如何在 Website 中配置

使用 `with_automation_scripts` 方法将脚本注入到爬虫配置中。同时需要配置 `ChromeEventTracker` 并将 `automation` 设置为 `true`。

### 示例代码片段：

```rust
use spider::configuration::{WebAutomation, ChromeEventTracker};
use spider::hashbrown::HashMap;

let mut automation_scripts = HashMap::new();

// 为匹配 "/login" 的页面定义操作序列
automation_scripts.insert(
    "/login".into(),
    Vec::from([
        WebAutomation::WaitFor("input[name='username']".into()),
        WebAutomation::Fill {
            selector: "input[name='username']".into(),
            value: "my_user".into(),
        },
        WebAutomation::Fill {
            selector: "input[name='password']".into(),
            value: "my_pass".into(),
        },
        WebAutomation::Click("button[type='submit']".into()),
        WebAutomation::Wait(2000), // 等待跳转
    ]),
);

let mut tracker = ChromeEventTracker::new(true, true);
tracker.automation = true;

let website = Website::new("https://example.com")
    .with_event_tracker(Some(tracker))
    .with_automation_scripts(Some(automation_scripts))
    .build()?;
```

## 3. 页面配置 API

- **`with_viewport(Option<Viewport>)`**: 设置浏览器视口大小（如桌面 1920x1080）。
- **`with_chrome_intercept(RequestInterceptConfiguration)`**: 开启请求拦截，这是执行自动化操作的前提。
- **`with_wait_for_idle_network(Option<WaitForIdleNetwork>)`**: 等待网络空闲后再开始操作。

## 4. 优势

1.  **声明式**: 只需定义“做什么”，不需要管理浏览器句柄。
2.  **路径匹配**: 可以针对不同的 URL 路径执行不同的逻辑。
3.  **集成度高**: 与 `spider` 的并发爬取、去重、限速等功能完美结合。

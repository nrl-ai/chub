---
name: package
description: "Selenium Python bindings for browser automation, WebDriver sessions, waits, actions, cookies, and remote grid usage"
metadata:
  languages: "python"
  versions: "4.41.0"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "selenium,python,webdriver,browser-automation,testing,e2e"
---

# Selenium Python Package Guide

## Golden Rule

Use the official `selenium` package and the W3C WebDriver APIs exposed by the Python bindings. For local development on Chrome, Firefox, or Edge, let Selenium Manager find or download the matching driver when possible, but remember that Selenium still needs the actual browser installed on the machine.

## Install

Pin the package version your project expects:

```bash
python -m pip install "selenium==4.41.0"
```

Common alternatives:

```bash
uv add "selenium==4.41.0"
poetry add "selenium==4.41.0"
```

You still need a supported browser installed locally. For the common Chrome, Firefox, and Edge flows, Selenium Manager handles driver discovery and download automatically.

## Initialize A Local Browser Session

Chrome or Chromium:

```python
from selenium import webdriver
from selenium.webdriver.chrome.options import Options

options = Options()
options.add_argument("--headless=new")
options.add_argument("--window-size=1440,900")

driver = webdriver.Chrome(options=options)

try:
    driver.get("https://www.selenium.dev/selenium/web/web-form.html")
    print(driver.title)
finally:
    driver.quit()
```

Firefox:

```python
from selenium import webdriver
from selenium.webdriver.firefox.options import Options

options = Options()
options.add_argument("-headless")

driver = webdriver.Firefox(options=options)

try:
    driver.get("https://www.selenium.dev/")
finally:
    driver.quit()
```

Useful option patterns:

- `options.add_argument(...)` for headless mode, window size, proxies, and browser flags
- `options.binary_location = "/path/to/browser"` when the browser binary is not on the default path
- separate option classes per browser: `chrome.options.Options`, `firefox.options.Options`, `edge.options.Options`

## Core Usage

### Navigate, locate, fill, and click

Prefer `By` locators plus explicit waits:

```python
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.support.ui import WebDriverWait

driver = webdriver.Chrome()

try:
    driver.get("https://www.selenium.dev/selenium/web/web-form.html")

    text_box = WebDriverWait(driver, 10).until(
        EC.visibility_of_element_located((By.NAME, "my-text"))
    )
    text_box.clear()
    text_box.send_keys("Selenium docs")

    submit_button = driver.find_element(By.CSS_SELECTOR, "button")
    submit_button.click()

    message = WebDriverWait(driver, 10).until(
        EC.visibility_of_element_located((By.ID, "message"))
    )
    print(message.text)
finally:
    driver.quit()
```

### Wait for page state instead of sleeping

Use `WebDriverWait(...).until(...)` with `expected_conditions` for elements, titles, alerts, frames, and URL changes:

```python
from selenium.webdriver.common.by import By
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.support.ui import WebDriverWait

wait = WebDriverWait(driver, 10)
submit = wait.until(EC.element_to_be_clickable((By.CSS_SELECTOR, "button")))
submit.click()

wait.until(EC.url_contains("submitted"))
```

Selenium supports implicit waits, but the official waits guidance warns that mixing implicit and explicit waits can produce longer and confusing timeouts. Prefer explicit waits for most test and automation code.

### Interact with complex gestures

Use `ActionChains` for hover, drag and drop, click-and-hold, and keyboard combinations:

```python
from selenium import webdriver
from selenium.webdriver.common.action_chains import ActionChains
from selenium.webdriver.common.by import By

driver = webdriver.Chrome()

try:
    driver.get("https://www.selenium.dev/selenium/web/mouse_interaction.html")

    draggable = driver.find_element(By.ID, "draggable")
    target = driver.find_element(By.ID, "droppable")

    ActionChains(driver).drag_and_drop(draggable, target).perform()
finally:
    driver.quit()
```

### Switch context when the page is not the top document

Common context switches agents miss:

- iframes: `driver.switch_to.frame(...)` before locating elements inside the frame
- top document: `driver.switch_to.default_content()`
- new tabs or windows: `driver.switch_to.window(driver.window_handles[-1])`
- alerts: `driver.switch_to.alert`

## Remote WebDriver And Grid

Use `webdriver.Remote(...)` when the browser runs outside the current machine, such as Selenium Grid, containers, or hosted browser providers:

```python
import os
from selenium import webdriver
from selenium.webdriver.chrome.options import Options

options = Options()
options.add_argument("--headless=new")

driver = webdriver.Remote(
    command_executor=os.environ["SELENIUM_REMOTE_URL"],
    options=options,
)

try:
    driver.get("https://www.selenium.dev/")
    print(driver.title)
finally:
    driver.quit()
```

Configuration notes:

- Keep the remote URL in an environment variable such as `SELENIUM_REMOTE_URL`
- Provider-specific credentials or vendor capabilities are usually attached through that remote URL or extra capabilities
- Match your browser options to the remote environment instead of assuming local defaults

Inference from the Selenium client-server model: Selenium itself does not have a package-level API key or auth step. The auth surface is normally the application under test or the remote WebDriver service you connect to.

## Cookies, Session State, And Login Flows

Selenium can persist and restore browser state with cookies:

```python
from selenium import webdriver

driver = webdriver.Chrome()

try:
    driver.get("https://example.com")
    driver.add_cookie({"name": "sessionid", "value": "abc123"})
    driver.refresh()
    print(driver.get_cookies())
finally:
    driver.quit()
```

Practical rules:

- load a page on the target domain before calling `add_cookie(...)`
- use cookies or normal UI login flows for application auth
- keep secrets and session tokens in environment variables or your test secret store, not inline in test code

## Common Pitfalls

- Selenium Manager handles drivers, not browser installation. A missing browser binary is still a setup failure.
- `time.sleep(...)` is a poor substitute for `WebDriverWait`; it makes tests slower and more flaky.
- Mixing implicit waits and explicit waits can create surprising timeout behavior.
- Headless flags are browser-specific. `--headless=new` is a Chrome or Chromium pattern, while Firefox commonly uses `-headless`.
- Elements inside frames, shadow DOM, or newly opened windows require an explicit context change.
- Avoid brittle absolute XPath selectors when stable IDs, names, labels, or CSS selectors exist.
- Local and remote browsers can behave differently on downloads, certificates, and sandboxing; reproduce failures in the same environment class.

## Version-Sensitive Notes For `selenium 4.41.0`

- PyPI metadata shows `requires_python >=3.10`.
- The official bindings and WebDriver documentation remain the right source for APIs and behavior, even though the Python API docs landing page still shows `4.40.0`.
- Selenium 4.x includes Selenium Manager, which changes the old setup advice from "manually download chromedriver/geckodriver first" to "try the built-in manager first, then fall back to explicit driver or browser configuration only when needed."

## Official Sources Used

- Selenium Python API docs: `https://www.selenium.dev/selenium/docs/api/py/`
- Selenium Python API module index: `https://www.selenium.dev/selenium/docs/api/py/api`
- Selenium first-script guide: `https://www.selenium.dev/documentation/webdriver/getting_started/first_script/`
- Selenium waits guidance: `https://www.selenium.dev/documentation/webdriver/waits/`
- Selenium Manager docs: `https://www.selenium.dev/documentation/selenium_manager/`
- Selenium remote WebDriver docs: `https://www.selenium.dev/documentation/webdriver/drivers/remote_webdriver/`
- Selenium actions API docs: `https://www.selenium.dev/documentation/webdriver/actions_api/`
- Selenium cookies docs: `https://www.selenium.dev/documentation/webdriver/interactions/cookies/`
- Selenium PyPI page: `https://pypi.org/project/selenium/`
- Selenium PyPI JSON API: `https://pypi.org/pypi/selenium/json`

> ## Documentation Index
> Fetch the complete documentation index at: https://code.claude.com/docs/llms.txt
> Use this file to discover all available pages before exploring further.

# 使用 hooks 自动化操作

> 当 Claude Code 编辑文件、完成任务或需要输入时自动运行 shell 命令。格式化代码、发送通知、验证命令并强制执行项目规则。

Hooks 是用户定义的 shell 命令，在 Claude Code 生命周期中的特定点执行。它们对 Claude Code 的行为提供确定性控制，确保某些操作始终发生，而不是依赖 LLM 选择运行它们。使用 hooks 来强制执行项目规则、自动化重复任务，并将 Claude Code 与现有工具集成。

对于需要判断而不是确定性规则的决策，你也可以使用 [基于提示的 hooks](#prompt-based-hooks) 或 [基于代理的 hooks](#agent-based-hooks)，它们使用 Claude 模型来评估条件。

有关扩展 Claude Code 的其他方式，请参阅 [skills](/docs/zh-CN/skills) 用于为 Claude 提供额外的指令和可执行命令，[subagents](/docs/zh-CN/sub-agents) 用于在隔离的上下文中运行任务，以及 [plugins](/docs/zh-CN/plugins) 用于打包要在项目间共享的扩展。

<Tip>
  本指南涵盖常见用例和入门方法。有关完整的事件架构、JSON 输入/输出格式和异步 hooks 和 MCP 工具 hooks 等高级功能，请参阅 [Hooks 参考](/docs/zh-CN/hooks)。
</Tip>

<h2 id="set-up-your-first-hook">
  设置你的第一个 hook
</h2>

要创建 hook，请将 `hooks` 块添加到 [设置文件](#configure-hook-location)。本演练创建一个桌面通知 hook，这样每当 Claude 等待你的输入而不是监视终端时，你都会收到警报。

<Steps>
  <Step title="将 hook 添加到你的设置">
    打开 `~/.claude/settings.json` 并添加一个 `Notification` hook。下面的示例使用 `osascript` 用于 macOS；有关 Linux 和 Windows 命令，请参阅 [在 Claude 需要输入时获得通知](#get-notified-when-claude-needs-input)。

    ```json theme={null}
    {
      "hooks": {
        "Notification": [
          {
            "matcher": "",
            "hooks": [
              {
                "type": "command",
                "command": "osascript -e 'display notification \"Claude Code needs your attention\" with title \"Claude Code\"'"
              }
            ]
          }
        ]
      }
    }
    ```

    如果你的设置文件已经有一个 `hooks` 键，请将 `Notification` 作为现有事件键的同级添加，而不是替换整个对象。每个事件名称是单个 `hooks` 对象内的一个键：

    ```json theme={null}
    {
      "hooks": {
        "PostToolUse": [
          {
            "matcher": "Edit|Write",
            "hooks": [{ "type": "command", "command": "jq -r '.tool_input.file_path' | xargs npx prettier --write" }]
          }
        ],
        "Notification": [
          {
            "matcher": "",
            "hooks": [{ "type": "command", "command": "osascript -e 'display notification \"Claude Code needs your attention\" with title \"Claude Code\"'" }]
          }
        ]
      }
    }
    ```

    你也可以通过在 CLI 中描述你想要的内容来要求 Claude 为你编写 hook。
  </Step>

  <Step title="验证配置">
    输入 `/hooks` 打开 hooks 浏览器。你将看到所有可用 hook 事件的列表，每个配置了 hooks 的事件旁边都有一个计数。选择 `Notification` 以确认你的新 hook 出现在列表中。选择 hook 会显示其详细信息：事件、匹配器、类型、源文件和命令。
  </Step>

  <Step title="测试 hook">
    按 `Esc` 返回 CLI。要求 Claude 做需要权限的事情，然后切换离开终端。你应该会收到桌面通知。
  </Step>
</Steps>

<Tip>
  `/hooks` 菜单是只读的。要添加、修改或删除 hooks，请直接编辑你的设置 JSON 或要求 Claude 进行更改。
</Tip>

<h2 id="what-you-can-automate">
  你可以自动化什么
</h2>

Hooks 让你在 Claude Code 生命周期中的关键点运行代码：编辑后格式化文件、在执行前阻止命令、在 Claude 需要输入时发送通知、在会话开始时注入上下文等。有关完整的 hook 事件列表，请参阅 [Hooks 参考](/docs/zh-CN/hooks#hook-lifecycle)。

每个示例都包含一个现成的配置块，你可以将其添加到 [设置文件](#configure-hook-location)。

有关运行单独模型审查并将发现反馈回会话的 hooks 的生产示例，请参阅 [`security-guidance` 插件如何与 Claude Code 集成](/docs/zh-CN/security-guidance#how-the-plugin-integrates-with-claude-code)。

<h3 id="get-notified-when-claude-needs-input">
  在 Claude 需要输入时获得通知
</h3>

每当 Claude 完成工作并需要你的输入时获得桌面通知，这样你可以切换到其他任务而无需检查终端。

此 hook 使用 `Notification` 事件，当 Claude 等待输入或权限时触发。下面的每个选项卡使用平台的原生通知命令。将其添加到 `~/.claude/settings.json`：

<Tabs>
  <Tab title="macOS">
    ```json theme={null}
    {
      "hooks": {
        "Notification": [
          {
            "matcher": "",
            "hooks": [
              {
                "type": "command",
                "command": "osascript -e 'display notification \"Claude Code needs your attention\" with title \"Claude Code\"'"
              }
            ]
          }
        ]
      }
    }
    ```

    <Accordion title="如果没有通知出现">
      `osascript` 通过内置的 Script Editor 应用程序路由通知。如果 Script Editor 没有通知权限，命令会静默失败，macOS 不会提示你授予它。在 Terminal 中运行一次以使 Script Editor 出现在你的通知设置中：

      ```bash theme={null}
      osascript -e 'display notification "test"'
      ```

      现在还不会出现任何内容。打开 **System Settings > Notifications**，在列表中找到 **Script Editor**，并打开 **Allow Notifications**。再次运行该命令以确认测试通知出现。
    </Accordion>
  </Tab>

  <Tab title="Linux">
    ```json theme={null}
    {
      "hooks": {
        "Notification": [
          {
            "matcher": "",
            "hooks": [
              {
                "type": "command",
                "command": "notify-send 'Claude Code' 'Claude Code needs your attention'"
              }
            ]
          }
        ]
      }
    }
    ```
  </Tab>

  <Tab title="Windows (PowerShell)">
    ```json theme={null}
    {
      "hooks": {
        "Notification": [
          {
            "matcher": "",
            "hooks": [
              {
                "type": "command",
                "command": "powershell.exe -Command \"[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); [System.Windows.Forms.MessageBox]::Show('Claude Code needs your attention', 'Claude Code')\""
              }
            ]
          }
        ]
      }
    }
    ```
  </Tab>
</Tabs>

空的 `matcher` 对所有通知类型触发。要仅在特定事件上触发，请将其设置为以下值之一：

| Matcher                | 触发时机                                                  |
| :--------------------- | :---------------------------------------------------- |
| `permission_prompt`    | Claude 需要你批准工具使用                                      |
| `idle_prompt`          | Claude 完成并等待你的下一个提示                                   |
| `auth_success`         | 身份验证完成                                                |
| `elicitation_dialog`   | MCP 服务器打开引导表单                                         |
| `elicitation_complete` | MCP 引导表单被提交或关闭                                        |
| `elicitation_response` | MCP 引导响应被发送回服务器                                       |
| `agent_needs_input`    | 后台会话开始等待你的输入。仅在 [agent view](/docs/zh-CN/agent-view) 打开时触发 |
| `agent_completed`      | 后台会话完成或失败。仅在 [agent view](/docs/zh-CN/agent-view) 打开时触发    |

`agent_needs_input` 和 `agent_completed` 匹配器需要 Claude Code v2.1.198 或更高版本。

输入 `/hooks` 并选择 `Notification` 以确认 hook 已注册。有关完整的事件架构，请参阅 [Notification 参考](/docs/zh-CN/hooks#notification)。

<h3 id="auto-format-code-after-edits">
  编辑后自动格式化代码
</h3>

在 Claude 编辑的每个文件上自动运行 [Prettier](https://prettier.io/)，以便格式保持一致而无需手动干预。

此 hook 使用带有 `Edit|Write` 匹配器的 `PostToolUse` 事件，因此它仅在文件编辑工具之后运行。该命令使用 [`jq`](https://jqlang.github.io/jq/) 提取编辑的文件路径并将其传递给 Prettier。将其添加到项目根目录中的 `.claude/settings.json`：

```json theme={null}
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "jq -r '.tool_input.file_path' | xargs npx prettier --write"
          }
        ]
      }
    ]
  }
}
```

在 Claude Code v2.1.191 或更高版本上，你也可以将匹配器写为 `Edit,Write`，因为在这些版本上 `|` 和 `,` 是工具名称匹配器的可互换列表分隔符。

<Note>
  本页上的 Bash 示例使用 `jq` 进行 JSON 解析。使用 `brew install jq`（macOS）、`apt-get install jq`（Debian 和 Ubuntu）安装它，或参阅 [`jq` 下载](https://jqlang.github.io/jq/download/)。
</Note>

<h3 id="block-edits-to-protected-files">
  阻止对受保护文件的编辑
</h3>

防止 Claude 修改敏感文件，如 `.env`、`package-lock.json` 或 `.git/` 中的任何内容。Claude 会收到解释编辑被阻止原因的反馈，因此它可以调整其方法。

此示例使用 hook 调用的单独脚本文件。该脚本根据受保护模式列表检查目标文件路径，并以代码 2 退出以阻止编辑。

<Steps>
  <Step title="创建 hook 脚本">
    将其保存到 `.claude/hooks/protect-files.sh`：

    ```bash theme={null}
    #!/bin/bash
    # protect-files.sh

    INPUT=$(cat)
    FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

    PROTECTED_PATTERNS=(".env" "package-lock.json" ".git/")

    for pattern in "${PROTECTED_PATTERNS[@]}"; do
      if [[ "$FILE_PATH" == *"$pattern"* ]]; then
        echo "Blocked: $FILE_PATH matches protected pattern '$pattern'" >&2
        exit 2
      fi
    done

    exit 0
    ```
  </Step>

  <Step title="使脚本可执行（macOS 和 Linux）">
    Hook 脚本必须可执行才能让 Claude Code 运行它们：

    ```bash theme={null}
    chmod +x .claude/hooks/protect-files.sh
    ```
  </Step>

  <Step title="注册 hook">
    将 `PreToolUse` hook 添加到 `.claude/settings.json`，在任何 `Edit` 或 `Write` 工具调用之前运行脚本：

    ```json theme={null}
    {
      "hooks": {
        "PreToolUse": [
          {
            "matcher": "Edit|Write",
            "hooks": [
              {
                "type": "command",
                "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/protect-files.sh"
              }
            ]
          }
        ]
      }
    }
    ```
  </Step>
</Steps>

<h3 id="re-inject-context-after-compaction">
  压缩后重新注入上下文
</h3>

当 Claude 的上下文窗口填满时，压缩会总结对话以释放空间。这可能会丢失重要细节。使用带有 `compact` 匹配器的 `SessionStart` hook 在每次压缩后重新注入关键上下文。

你的命令写入 stdout 的任何文本都会添加到 Claude 的上下文中。此示例提醒 Claude 项目约定和最近的工作。将其添加到项目根目录中的 `.claude/settings.json`：

```json theme={null}
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "compact",
        "hooks": [
          {
            "type": "command",
            "command": "echo 'Reminder: use Bun, not npm. Run bun test before committing. Current sprint: auth refactor.'"
          }
        ]
      }
    ]
  }
}
```

你可以用任何产生动态输出的命令替换 `echo`，如 `git log --oneline -5` 来显示最近的提交。有关在每个会话开始时注入上下文，请考虑改用 [CLAUDE.md](/docs/zh-CN/memory)。有关环境变量，请参阅参考中的 [`CLAUDE_ENV_FILE`](/docs/zh-CN/hooks#persist-environment-variables)。

<h3 id="audit-configuration-changes">
  审计配置更改
</h3>

跟踪会话期间设置或 skills 文件何时更改。`ConfigChange` 事件在外部进程或编辑器修改配置文件时触发，因此你可以记录更改以进行合规性检查或阻止未授权的修改。

此示例将每个更改附加到审计日志。将其添加到 `~/.claude/settings.json`：

```json theme={null}
{
  "hooks": {
    "ConfigChange": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "jq -c '{timestamp: now | todate, source: .source, file: .file_path}' >> ~/claude-config-audit.log"
          }
        ]
      }
    ]
  }
}
```

匹配器按配置类型过滤：`user_settings`、`project_settings`、`local_settings`、`policy_settings` 或 `skills`。要阻止更改生效，以代码 2 退出或返回 `{"decision": "block"}`。有关完整的输入架构，请参阅 [ConfigChange 参考](/docs/zh-CN/hooks#configchange)。

<h3 id="reload-environment-when-directory-or-files-change">
  当目录或文件更改时重新加载环境
</h3>

某些项目根据你所在的目录设置不同的环境变量。[direnv](https://direnv.net/) 之类的工具在你的 shell 中自动执行此操作，但 Claude 的 Bash 工具不会自动拾取这些更改。

配对 `SessionStart` hook 和 `CwdChanged` hook 可以解决这个问题。`SessionStart` 加载你启动时所在目录的变量，`CwdChanged` 在 Claude 每次更改目录时重新加载它们。两者都写入 `CLAUDE_ENV_FILE`，Claude Code 在每个 Bash 命令之前作为脚本前导运行。将其添加到 `~/.claude/settings.json`：

```json theme={null}
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "direnv export bash > \"$CLAUDE_ENV_FILE\""
          }
        ]
      }
    ],
    "CwdChanged": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "direnv export bash > \"$CLAUDE_ENV_FILE\""
          }
        ]
      }
    ]
  }
}
```

在每个包含 `.envrc` 的目录中运行一次 `direnv allow`，以便 direnv 被允许加载它。如果你使用 devbox 或 nix 而不是 direnv，相同的模式适用于 `devbox shellenv` 或 `devbox global shellenv` 代替 `direnv export bash`。

要对特定文件而不是每个目录更改做出反应，请使用 `FileChanged` 和 `matcher` 列出要监视的文件名，用 `|` 分隔。要构建监视列表，Claude Code 将此值分割为文字文件名而不是作为正则表达式进行评估。有关当文件更改时相同值如何也过滤哪些 hook 组运行，请参阅 [FileChanged](/docs/zh-CN/hooks#filechanged)。此示例监视工作目录中 `.envrc` 和 `.env` 的更改：

```json theme={null}
{
  "hooks": {
    "FileChanged": [
      {
        "matcher": ".envrc|.env",
        "hooks": [
          {
            "type": "command",
            "command": "direnv export bash > \"$CLAUDE_ENV_FILE\""
          }
        ]
      }
    ]
  }
}
```

有关输入架构、`watchPaths` 输出和 `CLAUDE_ENV_FILE` 详情，请参阅 [CwdChanged](/docs/zh-CN/hooks#cwdchanged) 和 [FileChanged](/docs/zh-CN/hooks#filechanged) 参考条目。

<h3 id="auto-approve-specific-permission-prompts">
  自动批准特定权限提示
</h3>

跳过你总是允许的工具调用的批准对话。此示例自动批准 `ExitPlanMode`，这是 Claude 在完成呈现计划并要求继续时调用的工具，因此你不会在每次计划准备好时被提示。

与上面的退出代码示例不同，自动批准需要你的 hook 将 JSON 决策写入 stdout。`PermissionRequest` hook 在 Claude Code 即将显示权限对话时触发，返回 `"behavior": "allow"` 代表你回答它。

匹配器将 hook 的范围限制为仅 `ExitPlanMode`，因此没有其他提示受到影响。将其添加到 `~/.claude/settings.json`：

```json theme={null}
{
  "hooks": {
    "PermissionRequest": [
      {
        "matcher": "ExitPlanMode",
        "hooks": [
          {
            "type": "command",
            "command": "echo '{\"hookSpecificOutput\": {\"hookEventName\": \"PermissionRequest\", \"decision\": {\"behavior\": \"allow\"}}}'"
          }
        ]
      }
    ]
  }
}
```

当 hook 批准时，Claude Code 退出计划模式并恢复进入计划模式之前处于活动状态的任何权限模式。成绩单显示"Allowed by PermissionRequest hook"，其中对话会出现。hook 路径始终保持当前对话：它无法清除上下文并以对话可以的方式启动新的实现会话。

要改为设置特定的权限模式，你的 hook 的输出可以包含一个 `updatedPermissions` 数组，其中包含 `setMode` 条目。`mode` 值是任何权限模式，如 `default`、`acceptEdits` 或 `bypassPermissions`，`destination: "session"` 仅将其应用于当前会话。

<Note>
  `bypassPermissions` 仅在会话已启动时应用，具有绕过模式可用：`--dangerously-skip-permissions`、`--permission-mode bypassPermissions`、`--allow-dangerously-skip-permissions` 或 `permissions.defaultMode: "bypassPermissions"` 在设置中，且未被 [`permissions.disableBypassPermissionsMode`](/docs/zh-CN/permissions#managed-settings) 禁用。它永远不会作为 `defaultMode` 持久化。
</Note>

要将会话切换到 `acceptEdits`，你的 hook 将此 JSON 写入 stdout：

```json theme={null}
{
  "hookSpecificOutput": {
    "hookEventName": "PermissionRequest",
    "decision": {
      "behavior": "allow",
      "updatedPermissions": [
        { "type": "setMode", "mode": "acceptEdits", "destination": "session" }
      ]
    }
  }
}
```

保持匹配器尽可能狭窄。匹配 `.*` 或留下匹配器为空会自动批准每个权限提示，包括文件写入和 shell 命令。有关完整的决策字段集，请参阅 [PermissionRequest 参考](/docs/zh-CN/hooks#permissionrequest-decision-control)。

<h2 id="how-hooks-work">
  Hooks 如何工作
</h2>

Hook 事件在 Claude Code 中的特定生命周期点触发。当事件触发时，所有匹配的 hooks 并行运行，相同的 hook 命令会自动去重。下表显示每个事件及其触发时间：

| Event                 | When it fires                                                                                                                                                                                                                                         |
| :-------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `SessionStart`        | When a session begins or resumes                                                                                                                                                                                                                      |
| `Setup`               | When you start Claude Code with `--init-only`, or with `--init` or `--maintenance` in `-p` mode. For one-time preparation in CI or scripts                                                                                                            |
| `UserPromptSubmit`    | When you submit a prompt, before Claude processes it                                                                                                                                                                                                  |
| `UserPromptExpansion` | When a user-typed command expands into a prompt, before it reaches Claude. Can block the expansion                                                                                                                                                    |
| `PreToolUse`          | Before a tool call executes. Can block it                                                                                                                                                                                                             |
| `PermissionRequest`   | When a tool call needs a permission decision                                                                                                                                                                                                          |
| `PermissionDenied`    | When auto mode denies a tool call, including denials without a classifier verdict. Use JSON `hookSpecificOutput.retry: true` to tell the model it may retry the denied tool call. Claude Code ignores `retry` when the classifier produced no verdict |
| `PostToolUse`         | After a tool call succeeds                                                                                                                                                                                                                            |
| `PostToolUseFailure`  | After a tool call fails                                                                                                                                                                                                                               |
| `PostToolBatch`       | After a full batch of parallel tool calls resolves, before the next model call                                                                                                                                                                        |
| `Notification`        | When Claude Code sends a notification                                                                                                                                                                                                                 |
| `MessageDisplay`      | While assistant message text is displayed                                                                                                                                                                                                             |
| `SubagentStart`       | When a subagent is spawned                                                                                                                                                                                                                            |
| `SubagentStop`        | When a subagent finishes                                                                                                                                                                                                                              |
| `TaskCreated`         | When a task is being created via `TaskCreate`                                                                                                                                                                                                         |
| `TaskCompleted`       | When a task is being marked as completed                                                                                                                                                                                                              |
| `Stop`                | When Claude finishes responding                                                                                                                                                                                                                       |
| `StopFailure`         | When the turn ends due to an API error                                                                                                                                                                                                                |
| `TeammateIdle`        | When an [agent team](/docs/en/agent-teams) teammate is about to go idle                                                                                                                                                                                    |
| `InstructionsLoaded`  | When a CLAUDE.md or `.claude/rules/*.md` file is loaded into context. Fires at session start and when files are lazily loaded during a session                                                                                                        |
| `ConfigChange`        | When a configuration file changes during a session                                                                                                                                                                                                    |
| `CwdChanged`          | When the working directory changes, for example when Claude executes a `cd` command. Useful for reactive environment management with tools like direnv                                                                                                |
| `DirectoryAdded`      | When a working directory is added mid-session via `/add-dir` or the SDK `register_repo_root` control request                                                                                                                                          |
| `FileChanged`         | When a watched file changes on disk. The `matcher` field specifies which filenames to watch                                                                                                                                                           |
| `WorktreeCreate`      | When a worktree is being created via `--worktree`, `isolation: "worktree"`, or for a background session. Replaces default git behavior                                                                                                                |
| `WorktreeRemove`      | When a worktree is being removed at session exit, when a subagent finishes, or when you delete a background session                                                                                                                                   |
| `PreCompact`          | Before context compaction                                                                                                                                                                                                                             |
| `PostCompact`         | After context compaction completes                                                                                                                                                                                                                    |
| `PreModelSwitch`      | Before Claude Code applies a model switch that you or a client requested. Can block the switch                                                                                                                                                        |
| `PostModelSwitch`     | After the session's model changes, including changes Claude Code makes on its own, such as restoring the model when you resume a session                                                                                                              |
| `Elicitation`         | When an MCP server requests user input during a tool call                                                                                                                                                                                             |
| `ElicitationResult`   | After a user responds to an MCP elicitation, before the response is sent back to the server                                                                                                                                                           |
| `SessionEnd`          | When a session terminates                                                                                                                                                                                                                             |

每个 hook 都有一个 `type` 来确定它如何运行。大多数 hooks 使用 `"type": "command"`，它运行 shell 命令。还有四种其他类型可用：

* `"type": "http"`：将事件数据 POST 到 URL。请参阅 [HTTP hooks](#http-hooks)。
* `"type": "mcp_tool"`：在已连接的 MCP 服务器上调用工具。请参阅 [MCP tool hooks](/docs/zh-CN/hooks#mcp-tool-hook-fields)。
* `"type": "prompt"`：单轮 LLM 评估。请参阅 [Prompt-based hooks](#prompt-based-hooks)。
* `"type": "agent"`：具有工具访问权限的多轮验证。Agent hooks 是实验性的，可能会改变。请参阅 [Agent-based hooks](#agent-based-hooks)。

<h3 id="combine-results-from-multiple-hooks">
  合并来自多个 hooks 的结果
</h3>

当多个 hooks 匹配同一事件时，每个 hook 的命令都会运行到完成，然后 Claude Code 合并结果。一个 hook 返回 `deny` 不会阻止兄弟 hooks 执行。不要依赖一个 hook 的 `deny` 来抑制另一个 hook 中的副作用。

所有匹配的 hooks 完成后，Claude Code 合并它们的输出。对于 `PreToolUse` 权限决策，最严格的答案获胜，顺序为 `deny`、`defer`、`ask`、`allow`。来自 `additionalContext` 的文本从每个 hook 保留并一起传递给 Claude。

下面的示例在 `Bash` 上注册两个 `PreToolUse` hooks。第一个将每个命令附加到日志文件并以 0 退出。第二个运行一个脚本，当命令包含 `rm -rf` 时以 2 退出以拒绝：

```json theme={null}
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "jq -r .tool_input.command >> ~/.claude/bash.log"
          },
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/block-rm-rf.sh"
          }
        ]
      }
    ]
  }
}
```

当 Claude 尝试运行 `rm -rf /tmp/build` 时，两个 hooks 并行执行。日志 hook 将命令写入 `~/.claude/bash.log` 并以 0 退出，这表示没有决策。防护栏 hook 以 2 退出，这拒绝了工具调用。拒绝获胜，所以 Claude Code 阻止命令并向 Claude 显示防护栏的 stderr。日志条目仍然被写入，因为日志 hook 已经运行。

<h3 id="read-input-and-return-output">
  读取输入并返回输出
</h3>

Hooks 通过 stdin、stdout、stderr 和退出代码与 Claude Code 通信。当事件触发时，Claude Code 将事件特定的数据作为 JSON 传递到脚本的 stdin。你的脚本读取该数据，完成其工作，并通过退出代码告诉 Claude Code 接下来要做什么。

<h4 id="hook-input">
  Hook 输入
</h4>

每个事件都包含常见字段，如 `session_id` 和 `cwd`，但每个事件类型添加不同的数据。例如，当 Claude 运行 Bash 命令时，`PreToolUse` hook 在 stdin 上接收类似以下内容：

```json theme={null}
{
  "session_id": "abc123",          // 此会话的唯一 ID
  "cwd": "/Users/sarah/myproject", // 事件触发时的工作目录
  "hook_event_name": "PreToolUse", // 哪个事件触发了此 hook
  "tool_name": "Bash",             // Claude 即将使用的工具
  "tool_input": {                  // Claude 传递给工具的参数
    "command": "npm test"          // 对于 Bash，这是 shell 命令
  }
}
```

你的脚本可以解析该 JSON 并对任何这些字段进行操作。`UserPromptSubmit` hooks 获取 `prompt` 文本，`SessionStart` hooks 获取 `source`（启动、恢复、清除、压缩），等等。有关共享字段，请参阅参考中的 [常见输入字段](/docs/zh-CN/hooks#common-input-fields)，以及每个事件的部分了解事件特定的架构。

<h4 id="hook-output">
  Hook 输出
</h4>

你的脚本通过写入 stdout 或 stderr 并以特定代码退出来告诉 Claude Code 接下来要做什么。以下 `PreToolUse` hook 阻止一个命令：

```bash theme={null}
#!/bin/bash
INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command')

if echo "$COMMAND" | grep -q "drop table"; then
  echo "Blocked: dropping tables is not allowed" >&2  # stderr 变成 Claude 的反馈
  exit 2 # exit 2 = 阻止操作
fi

exit 0  # exit 0 = 没有决策；正常权限流程适用
```

退出代码确定接下来会发生什么：

* **退出 0**：hook 报告没有异议，操作正常进行。对于 `PreToolUse` hook，这不会批准工具调用：正常的 [权限流程](/docs/zh-CN/permissions) 仍然适用。对于 `UserPromptSubmit`、`UserPromptExpansion` 和 `SessionStart` hooks，你写入 stdout 的任何内容都会添加到 Claude 的上下文中。
* **退出 2**：操作被阻止。写入原因到 stderr，Claude 会收到它作为反馈，以便它可以调整。某些事件无法被阻止：对于 `SessionStart`、`Setup`、`Notification` 和其他事件，退出 2 向用户显示 stderr，执行继续。有关每个事件的退出代码 2 行为的完整列表，请参阅 [每个事件的退出代码 2 行为](/docs/zh-CN/hooks#exit-code-2-behavior-per-event)。
* **任何其他退出代码**：操作继续。成绩单显示 `<hook name> hook error` 通知，后跟 stderr 的第一行；完整的 stderr 进入 [调试日志](/docs/zh-CN/hooks#debug-hooks)。

<h4 id="structured-json-output">
  结构化 JSON 输出
</h4>

退出代码只让你阻止或保持沉默。为了获得更多控制，退出 0 并改为将 JSON 对象打印到 stdout。

<Note>
  使用退出 2 以 stderr 消息阻止，或使用 JSON 退出 0 以获得结构化控制。不要混合它们：Claude Code 在你退出 2 时忽略 JSON。
</Note>

例如，`PreToolUse` hook 可以拒绝工具调用并告诉 Claude 为什么，或将其升级给用户以获得批准：

```json theme={null}
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "deny",
    "permissionDecisionReason": "Use rg instead of grep for better performance"
  }
}
```

使用 `"deny"`，Claude Code 取消工具调用并将 `permissionDecisionReason` 反馈给 Claude。这些 `permissionDecision` 值特定于 `PreToolUse`：

* `"allow"`：跳过交互式权限提示。拒绝和询问规则，包括企业托管拒绝列表，仍然适用，以及你的组织设置为 `ask` 的 [连接器工具](/docs/zh-CN/mcp#organization-controls-on-connector-tools) 和标记为 [`requiresUserInteraction`](/docs/zh-CN/mcp#require-approval-for-a-specific-tool) 的 MCP 工具
* `"deny"`：取消工具调用并将原因发送给 Claude
* `"ask"`：照常向用户显示权限提示

第四个值 `"defer"` 在 [非交互模式](/docs/zh-CN/headless) 中使用 `-p` 标志时可用。它以保留的工具调用退出进程，以便 Agent SDK 包装器可以收集输入并恢复。请参阅参考中的 [延迟工具调用以供稍后使用](/docs/zh-CN/hooks#defer-a-tool-call-for-later)。

返回 `"allow"` 跳过交互式提示但不覆盖 [权限规则](/docs/zh-CN/permissions#manage-permissions)。如果拒绝规则与工具调用匹配，即使你的 hook 返回 `"allow"`，调用也会被阻止。如果询问规则匹配，用户仍然会被提示，以及你的组织设置为 `ask` 的 [连接器工具](/docs/zh-CN/mcp#organization-controls-on-connector-tools) 和标记为 [`requiresUserInteraction`](/docs/zh-CN/mcp#require-approval-for-a-specific-tool) 的 MCP 工具。这意味着来自任何设置范围的拒绝规则，包括 [托管设置](/docs/zh-CN/settings#settings-files)，总是优先于 hook 批准。

其他事件使用不同的决策模式。例如，`PostToolUse` 和 `Stop` hooks 使用顶级 `decision: "block"` 字段，而 `PermissionRequest` 使用 `hookSpecificOutput.decision.behavior`。有关按事件的完整分解，请参阅参考中的 [摘要表](/docs/zh-CN/hooks#decision-control)。

对于 `UserPromptSubmit` hooks，改用 `hookSpecificOutput.additionalContext` 将文本注入到 Claude 的上下文中。将 `additionalContext` 嵌套在 `hookSpecificOutput` 内；如果你将其放在 JSON 的顶级，Claude Code 会默默忽略它。例如，此输出将当前分支状态添加到每个提示：

```json theme={null}
{
  "hookSpecificOutput": {
    "hookEventName": "UserPromptSubmit",
    "additionalContext": "Current branch: release-42. Deploy freeze until Friday."
  }
}
```

有关完整的输出形状，包括阻止提示和设置会话标题，请参阅 [UserPromptSubmit 决策控制](/docs/zh-CN/hooks#userpromptsubmit-decision-control)。

基于 prompt 的 hooks（`type: "prompt"`）处理输出的方式不同：请参阅 [Prompt-based hooks](#prompt-based-hooks)。

<h3 id="filter-hooks-with-matchers">
  使用匹配器过滤 hooks
</h3>

没有匹配器，hook 会在其事件的每次出现时触发。匹配器让你缩小范围。例如，如果你只想在文件编辑后运行格式化程序（而不是在每个工具调用后），将匹配器添加到你的 `PostToolUse` hook：

```json theme={null}
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          { "type": "command", "command": "prettier --write ..." }
        ]
      }
    ]
  }
}
```

`"Edit|Write"` 匹配器仅在 Claude 使用 `Edit` 或 `Write` 工具时触发，而不是在它使用 `Bash`、`Read` 或任何其他工具时触发。在 Claude Code v2.1.191 或更高版本上，逗号以相同的方式分隔替代项，所以 `"Edit, Write"` 是等效的。请参阅 [匹配器模式](/docs/zh-CN/hooks#matcher-patterns) 了解纯名称和正则表达式如何被评估。

<Note>
  Claude 也可以通过 `Bash` 工具运行 shell 命令来创建或修改文件。如果你的 hook 必须看到每个文件更改，例如用于合规性扫描或审计日志，添加一个 [`Stop`](/docs/zh-CN/hooks#stop) hook，它每轮扫描一次工作树。为了获得每次调用的覆盖，也匹配 `Bash` 并让你的脚本使用 `git status --porcelain` 列出修改和未跟踪的文件。
</Note>

每个事件类型在特定字段上匹配：

| 事件                                                                                                                                                     | 匹配器过滤的内容                                              | 示例匹配器值                                                                                                                                                                     |
| :----------------------------------------------------------------------------------------------------------------------------------------------------- | :---------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `PreToolUse`、`PostToolUse`、`PostToolUseFailure`、`PermissionRequest`、`PermissionDenied`                                                                 | 工具名称                                                  | `Bash`、`Edit\|Write`、`mcp__.*`                                                                                                                                             |
| `SessionStart`                                                                                                                                         | 会话如何启动                                                | `startup`、`resume`、`clear`、`compact`                                                                                                                                       |
| `Setup`                                                                                                                                                | 哪个 CLI 标志触发了设置                                        | `init`、`maintenance`                                                                                                                                                       |
| `SessionEnd`                                                                                                                                           | 会话为什么结束                                               | `clear`、`resume`、`logout`、`prompt_input_exit`、`bypass_permissions_disabled`、`other`                                                                                        |
| `Notification`                                                                                                                                         | 通知类型                                                  | `permission_prompt`、`idle_prompt`、`auth_success`、`elicitation_dialog`、`elicitation_complete`、`elicitation_response`、`agent_needs_input`、`agent_completed`                  |
| `SubagentStart`                                                                                                                                        | 代理类型                                                  | `general-purpose`、`Explore`、`Plan` 或自定义代理名称                                                                                                                                |
| `PreCompact`、`PostCompact`                                                                                                                             | 什么触发了压缩                                               | `manual`、`auto`                                                                                                                                                            |
| `SubagentStop`                                                                                                                                         | 代理类型                                                  | 与 `SubagentStart` 相同的值                                                                                                                                                     |
| `ConfigChange`                                                                                                                                         | 配置源                                                   | `user_settings`、`project_settings`、`local_settings`、`policy_settings`、`skills`                                                                                             |
| `StopFailure`                                                                                                                                          | 错误类型                                                  | `rate_limit`、`overloaded`、`authentication_failed`、`oauth_org_not_allowed`、`billing_error`、`invalid_request`、`model_not_found`、`server_error`、`max_output_tokens`、`unknown` |
| `InstructionsLoaded`                                                                                                                                   | 加载原因                                                  | `session_start`、`nested_traversal`、`path_glob_match`、`include`、`compact`                                                                                                   |
| `Elicitation`                                                                                                                                          | MCP 服务器名称                                             | 你配置的 MCP 服务器名称                                                                                                                                                             |
| `ElicitationResult`                                                                                                                                    | MCP 服务器名称                                             | 与 `Elicitation` 相同的值                                                                                                                                                       |
| `FileChanged`                                                                                                                                          | 文字文件名来监视（请参阅 [FileChanged](/docs/zh-CN/hooks#filechanged)） | `.envrc\|.env`                                                                                                                                                             |
| `UserPromptExpansion`                                                                                                                                  | 命令名称                                                  | 你的 skill 或命令名称                                                                                                                                                             |
| `UserPromptSubmit`、`PostToolBatch`、`Stop`、`TeammateIdle`、`TaskCreated`、`TaskCompleted`、`WorktreeCreate`、`WorktreeRemove`、`CwdChanged`、`MessageDisplay` | 不支持匹配器                                                | 始终在每次出现时触发                                                                                                                                                                 |

下面的选项卡显示不同事件类型上的更多匹配器示例。

<Tabs>
  <Tab title="记录每个 Bash 命令">
    仅匹配 `Bash` 工具调用并将每个命令记录到文件。`PostToolUse` 事件在命令完成后触发，因此 `tool_input.command` 包含运行的内容。hook 在 stdin 上接收事件数据作为 JSON，`jq -r '.tool_input.command'` 仅提取命令字符串，`>>` 将其附加到日志文件：

    ```json theme={null}
    {
      "hooks": {
        "PostToolUse": [
          {
            "matcher": "Bash",
            "hooks": [
              {
                "type": "command",
                "command": "jq -r '.tool_input.command' >> ~/.claude/command-log.txt"
              }
            ]
          }
        ]
      }
    }
    ```
  </Tab>

  <Tab title="匹配 MCP 工具">
    MCP 工具使用与内置工具不同的命名约定：`mcp__<server>__<tool>`，其中 `<server>` 是 MCP 服务器名称，`<tool>` 是它提供的工具。例如，`mcp__github__search_repositories` 或 `mcp__filesystem__read_file`。使用正则表达式匹配器来针对来自特定服务器的所有工具，或使用 `mcp__.*__write.*` 之类的模式跨服务器匹配。有关完整的示例列表，请参阅参考中的 [匹配 MCP 工具](/docs/zh-CN/hooks#match-mcp-tools)。

    下面的命令使用 `jq` 从 hook 的 JSON 输入中提取工具名称，并将其写入 stderr。将其写入 stderr 保持 stdout 清洁以用于 JSON 输出，并将消息发送到 [调试日志](/docs/zh-CN/hooks#debug-hooks)：

    ```json theme={null}
    {
      "hooks": {
        "PreToolUse": [
          {
            "matcher": "mcp__github__.*",
            "hooks": [
              {
                "type": "command",
                "command": "echo \"GitHub tool called: $(jq -r '.tool_name')\" >&2"
              }
            ]
          }
        ]
      }
    }
    ```
  </Tab>

  <Tab title="在会话结束时清理">
    `SessionEnd` 事件支持会话结束原因的匹配器。此 hook 仅在 `clear` 时触发（当你运行 `/clear` 时），而不是在正常退出时：

    ```json theme={null}
    {
      "hooks": {
        "SessionEnd": [
          {
            "matcher": "clear",
            "hooks": [
              {
                "type": "command",
                "command": "rm -f /tmp/claude-scratch-*.txt"
              }
            ]
          }
        ]
      }
    }
    ```
  </Tab>
</Tabs>

有关完整的匹配器语法，请参阅 [Hooks 参考](/docs/zh-CN/hooks#configuration)。

<h4 id="filter-by-tool-name-and-arguments-with-the-if-field">
  使用 `if` 字段按工具名称和参数过滤
</h4>

`if` 字段使用 [权限规则语法](/docs/zh-CN/permissions) 按工具名称和参数一起过滤 hooks，因此 hook 进程仅在工具调用匹配时生成。这超越了 `matcher`，它仅在工具名称级别按组过滤。

例如，这个配置仅在 Claude 使用 `git` 命令而不是所有 Bash 命令时运行 hook：

```json theme={null}
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "if": "Bash(git *)",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/check-git-policy.sh"
          }
        ]
      }
    ]
  }
}
```

你的 hook 命令是否运行取决于你的 `if` 模式的形状和 Claude 正在调用的 Bash 命令：

| `if` 模式            | Bash 命令                | Hook 运行？ | 为什么                                     |
| :----------------- | :--------------------- | :------- | :-------------------------------------- |
| `Bash(git *)`      | `git push`             | 是        | 命令名称匹配                                  |
| `Bash(git *)`      | `npm test && git push` | 是        | 每个子命令都被检查；`git push` 匹配                 |
| `Bash(git *)`      | `echo $(git log)`      | 是        | `$()` 和反引号内的命令被检查；`git log` 匹配          |
| `Bash(git *)`      | `echo $(date)`         | 否        | 没有子命令匹配 `git *`                         |
| `Bash(git push *)` | `echo $(date)`         | 是        | 指定超过命令名称的模式在 `$()`、反引号或 `$VAR` 上运行 hook |

当 Bash 命令无法解析时，过滤器也会失败开放，无论如何都会运行你的 hook。因为过滤器是尽力而为的，使用 [权限系统](/docs/zh-CN/permissions) 而不是 hook 来强制执行硬允许或拒绝。

`if` 字段接受与权限规则相同的模式：`"Bash(git *)"`、`"Edit(*.ts)"` 等。要匹配多个工具名称，使用单独的处理程序，每个都有自己的 `if` 值，或在 `matcher` 级别匹配，其中支持管道交替。

`if` 仅适用于工具事件：`PreToolUse`、`PostToolUse`、`PostToolUseFailure`、`PermissionRequest` 和 `PermissionDenied`。将其添加到任何其他事件会阻止 hook 运行。

<h3 id="configure-hook-location">
  配置 hook 位置
</h3>

你添加 hook 的位置决定了其范围：

| 位置                                                              | 范围                      | 可共享                             |
| :-------------------------------------------------------------- | :---------------------- | :------------------------------ |
| `~/.claude/settings.json`                                       | 所有你的项目                  | 否，本地到你的机器                       |
| `.claude/settings.json`                                         | 单个项目                    | 是，可以提交到仓库                       |
| `.claude/settings.local.json`                                   | 单个项目                    | 否，gitignored 当 Claude Code 创建它时 |
| 托管策略设置                                                          | 组织范围                    | 是，管理员控制                         |
| [Plugin](/docs/zh-CN/plugins) `hooks/hooks.json`                     | 启用插件时                   | 是，与插件捆绑                         |
| [Skill](/docs/zh-CN/skills) 或 [agent](/docs/zh-CN/sub-agents) frontmatter | 当 skill 或 agent 处于活动状态时 | 是，在组件文件中定义                      |

在 Claude Code 中运行 [`/hooks`](/docs/zh-CN/hooks#the-%2Fhooks-menu) 以浏览所有按事件分组的配置 hooks。

要禁用 hooks，在设置文件中设置 `"disableAllHooks": true`。托管设置中配置的 Hooks 仍然运行，除非 `disableAllHooks` 也在那里设置。

如果你在 Claude Code 运行时直接编辑设置文件，文件监视器通常会自动拾取 hook 更改。

<h2 id="prompt-based-hooks">
  基于提示的 hooks
</h2>

对于需要判断而不是确定性规则的决策，使用 `type: "prompt"` hooks。Claude Code 不运行 shell 命令，而是将你的提示和 hook 的输入数据发送到 Claude 模型（默认为 Haiku）来做出决策。如果你需要更多功能，可以使用 `model` 字段指定不同的模型。

模型的唯一工作是返回一个是/否决策作为 JSON：

* `"ok": true`：操作继续
* `"ok": false`：发生的情况取决于事件：
  * `Stop` 和 `SubagentStop`：`reason` 被反馈给 Claude，以便它继续工作
  * `PreToolUse`：工具调用被拒绝，`reason` 作为工具错误返回给 Claude，以便它可以调整并继续
  * `PostToolUse`、`PostToolBatch`、`UserPromptSubmit` 和 `UserPromptExpansion`：回合结束，`reason` 在聊天中显示为警告行

此示例使用 `Stop` hook 询问模型是否所有请求的任务都已完成。如果模型返回 `"ok": false`，Claude 继续工作并使用 `reason` 作为其下一条指令：

```json theme={null}
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "prompt",
            "prompt": "Check if all tasks are complete. If not, respond with {\"ok\": false, \"reason\": \"what remains to be done\"}."
          }
        ]
      }
    ]
  }
}
```

有关完整的配置选项，请参阅参考中的 [基于提示的 hooks](/docs/zh-CN/hooks#prompt-based-hooks)。

<h2 id="agent-based-hooks">
  基于代理的 hooks
</h2>

<Warning>
  代理 hooks 是实验性的。行为和配置可能在未来版本中改变。对于生产工作流，更倾向于 [命令 hooks](/docs/zh-CN/hooks#command-hook-fields)。
</Warning>

当验证需要检查文件或运行命令时，使用 `type: "agent"` hooks。与只进行单个 LLM 调用的提示 hooks 不同，代理 hooks 生成一个 subagent，它可以读取文件、搜索代码和使用其他工具来验证条件，然后返回决策。

代理 hooks 使用与提示 hooks 相同的 `"ok"` / `"reason"` 响应格式，但默认超时更长（60 秒）和最多 50 个工具使用轮次。

此示例验证在允许 Claude 停止之前测试通过：

```json theme={null}
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "agent",
            "prompt": "Verify that all unit tests pass. Run the test suite and check the results. $ARGUMENTS",
            "timeout": 120
          }
        ]
      }
    ]
  }
}
```

当 hook 输入数据本身足以做出决策时使用提示 hooks。当你需要根据代码库的实际状态验证某些内容时使用代理 hooks。

有关完整的配置选项，请参阅参考中的 [基于代理的 hooks](/docs/zh-CN/hooks#agent-based-hooks)。

<h2 id="http-hooks">
  HTTP hooks
</h2>

使用 `type: "http"` hooks 将事件数据 POST 到 HTTP 端点，而不是运行 shell 命令。端点接收命令 hook 在 stdin 上接收的相同 JSON，并使用相同的 JSON 格式通过 HTTP 响应体返回结果。

HTTP hooks 在你想要 web 服务器、云函数或外部服务处理 hook 逻辑时很有用：例如，一个跨团队记录工具使用事件的共享审计服务。

此示例将每个工具使用 POST 到本地日志服务：

```json theme={null}
{
  "hooks": {
    "PostToolUse": [
      {
        "hooks": [
          {
            "type": "http",
            "url": "http://localhost:8080/hooks/tool-use",
            "headers": {
              "Authorization": "Bearer $MY_TOKEN"
            },
            "allowedEnvVars": ["MY_TOKEN"]
          }
        ]
      }
    ]
  }
}
```

端点应使用与命令 hooks 相同的 [输出格式](/docs/zh-CN/hooks#json-output) 返回 JSON 响应体。要阻止工具调用，返回 2xx 响应，包含适当的 `hookSpecificOutput` 字段。HTTP 状态代码本身无法阻止操作。

标头值支持使用 `$VAR_NAME` 或 `${VAR_NAME}` 语法的环境变量插值。仅解析 `allowedEnvVars` 数组中列出的变量；所有其他 `$VAR` 引用保持为空。

有关完整的配置选项和响应处理，请参阅参考中的 [HTTP hooks](/docs/zh-CN/hooks#http-hook-fields)。

<h2 id="limitations-and-troubleshooting">
  限制和故障排除
</h2>

<h3 id="limitations">
  限制
</h3>

设计 hooks 时请记住这些约束：

* 命令 hooks 仅通过 stdout、stderr 和退出代码通信。它们无法触发 `/` 命令或工具调用。通过 `additionalContext` 返回的文本被注入为 Claude 作为纯文本读取的系统提醒。HTTP hooks 改为通过响应体通信。
* Hook 超时因类型而异。通过 `timeout` 字段（以秒为单位）按 hook 覆盖。
  * `command`、`http`、`mcp_tool`：10 分钟。`UserPromptSubmit` 将这些降低到 30 秒，`MessageDisplay` 将这些降低到 10 秒。
  * `prompt`：30 秒。
  * `agent`：60 秒。
* `PostToolUse` hooks 无法撤销操作，因为工具已经执行。
* `PermissionRequest` hooks 不在 [非交互模式](/docs/zh-CN/headless)（带 `-p` 标志）中触发。对于自动化权限决策，使用 `PreToolUse` hooks。
* `Stop` hooks 在 Claude 完成响应时触发，而不仅仅在任务完成时。它们不在用户中断时触发。API 错误触发 [StopFailure](/docs/zh-CN/hooks#stopfailure) 代替。
* 当多个 `PreToolUse` hooks 返回 [`updatedInput`](/docs/zh-CN/hooks#pretooluse) 来重写工具的参数时，最后完成的获胜。由于 hooks 并行运行，顺序是非确定性的。避免有多个 hook 修改同一工具的输入。

<h3 id="hooks-and-permission-modes">
  Hooks 和权限模式
</h3>

`PreToolUse` hooks 在任何权限模式检查之前触发。返回 `permissionDecision: "deny"` 的 hook 会阻止工具，即使在 `bypassPermissions` 模式或使用 `--dangerously-skip-permissions` 时也是如此。这让你强制执行用户无法通过更改其权限模式来绕过的策略。

反面不成立：返回 `"allow"` 的 hook 不会绕过来自设置的拒绝规则，它也无法抑制你的组织设置为 `ask` 的连接器工具的提示 [](/docs/zh-CN/mcp#organization-controls-on-connector-tools) 或标记为 [`requiresUserInteraction`](/docs/zh-CN/mcp#require-approval-for-a-specific-tool) 的 MCP 工具。Hooks 可以收紧限制，但不能放松它们超过权限规则允许的范围。

<h3 id="hook-not-firing">
  Hook 未触发
</h3>

Hook 已配置但从不执行。

* 运行 `/hooks` 并确认 hook 出现在正确的事件下
* 检查匹配器模式是否与工具名称完全匹配。匹配器区分大小写
* 验证你是否触发了正确的事件类型：`PreToolUse` 在工具执行前触发，`PostToolUse` 在之后触发
* 如果在非交互模式（带 `-p` 标志）中使用 `PermissionRequest` hooks，改用 `PreToolUse`

<h3 id="hook-error-in-output">
  Hook 输出中的错误
</h3>

你在成绩单中看到类似"PreToolUse hook error: ..."的消息。

* 你的脚本意外以非零代码退出。通过管道传递示例 JSON 来手动测试它：
  ```bash theme={null}
  echo '{"tool_name":"Bash","tool_input":{"command":"ls"}}' | ./my-hook.sh
  echo $?  # 检查退出代码
  ```
* 如果你看到"command not found"，使用绝对路径或 `${CLAUDE_PROJECT_DIR}` 来引用脚本。为了完全避免 shell 引用，添加 `"args": []` 来切换到 [exec 形式](/docs/zh-CN/hooks#exec-form-and-shell-form)，它直接生成脚本而不使用 shell
* 如果你看到"jq: command not found"，安装 `jq` 或使用 Python/Node.js 进行 JSON 解析
* 如果脚本根本没有运行，使其可执行：`chmod +x ./my-hook.sh`

<h3 id="/hooks-shows-no-hooks-configured">
  `/hooks` 显示未配置 hooks
</h3>

你编辑了设置文件但 hooks 不出现在菜单中。

* 文件编辑通常会自动拾取。如果几秒钟后它们还没有出现，文件监视器可能错过了更改：重新启动你的会话以强制重新加载。
* 验证你的 JSON 有效：不允许尾随逗号和注释
* 确认设置文件在正确的位置：`.claude/settings.json` 用于项目 hooks，`~/.claude/settings.json` 用于全局 hooks

<h3 id="stop-hook-hits-the-block-cap">
  Stop hook 达到阻止上限
</h3>

Claude 继续工作而不是停止，然后以警告结束该轮，表示 Stop hook 连续阻止了太多次。

Claude Code 在 Stop hook 连续阻止 8 次而没有进展后会覆盖它。你的 hook 脚本需要检查它是否已经触发了继续。从 JSON 输入中解析 `stop_hook_active` 字段，如果为 `true` 则提前退出：

```bash theme={null}
#!/bin/bash
INPUT=$(cat)
if [ "$(echo "$INPUT" | jq -r '.stop_hook_active')" = "true" ]; then
  exit 0  # 允许 Claude 停止
fi
# ... 你的 hook 逻辑的其余部分
```

如果你的 hook 合理地需要超过八次迭代才能收敛，使用 [`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`](/docs/zh-CN/env-vars) 提高上限。

<h3 id="json-validation-failed">
  JSON 验证失败
</h3>

Claude Code 显示 JSON 解析错误，即使你的 hook 脚本输出有效的 JSON。

当 Claude Code 运行 shell 形式的命令 hook（没有 `args` 的）时，它在 macOS 和 Linux 上生成 `sh -c`，或在 Windows 上生成 Git Bash。这个 shell 是非交互式的，但 Git Bash 和某些配置（例如 `BASH_ENV` 指向 `~/.bashrc`）仍然会源你的配置文件。如果该配置文件包含无条件的 `echo` 语句，输出会被添加到你的 hook 的 JSON 前面：

```text theme={null}
Shell ready on arm64
{"decision": "block", "reason": "Not allowed"}
```

Claude Code 尝试将其解析为 JSON 并失败。要修复此问题，在你的 shell 配置文件中包装 echo 语句，使其仅在交互式 shell 中运行：

```bash theme={null}
# 在 ~/.zshrc 或 ~/.bashrc 中
if [[ $- == *i* ]]; then
  echo "Shell ready"
fi
```

`$-` 变量包含 shell 标志，`i` 表示交互式。Hooks 在非交互式 shell 中运行，因此 echo 被跳过。

<h3 id="debug-techniques">
  调试技术
</h3>

成绩单视图，使用 `Ctrl+O` 切换，显示每个触发的 hook 的单行摘要：成功是无声的，阻止错误显示 stderr，非阻止错误显示 `<hook name> hook error` 通知，后跟 stderr 的第一行。

有关完整的执行详情，包括哪些 hooks 匹配、它们的退出代码、stdout 和 stderr，请阅读调试日志。使用 `claude --debug-file /tmp/claude.log` 启动 Claude Code 以写入已知路径，然后在另一个终端中 `tail -f /tmp/claude.log`。如果你启动时没有该标志，在会话中运行 `/debug` 以启用日志记录并找到日志路径。

<h2 id="learn-more">
  了解更多
</h2>

* [Hooks 参考](/docs/zh-CN/hooks)：完整的事件架构、JSON 输出格式、异步 hooks 和 MCP 工具 hooks
* [安全考虑](/docs/zh-CN/hooks#security-considerations)：在共享或生产环境中部署 hooks 之前查看
* [Bash 命令验证器示例](https://github.com/anthropics/claude-code/blob/main/examples/hooks/bash_command_validator_example.py)：完整的参考实现

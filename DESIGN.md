# DESIGN.md

## Purpose

This document defines the public design rules for the PAO terminal user interface. PAO is a macOS-first terminal AI coding agent, and its TUI must prioritize clarity, safety, auditability, and keyboard-first operation.

The implementation target is Rust with `ratatui` and `crossterm`.

## Design Principles

- Use color as a supporting signal, not as the only signal.
- Define status meaning through semantic tokens.
- UI code must reference semantic tokens, not raw color values.
- Use the terminal default background for the main screen.
- Use the terminal default foreground for body text.
- Communicate important states with at least two signals among color, label, glyph, position, and border style.
- Dangerous actions must include explicit text such as `destructive`, `requires approval`, or `blocked`.
- The UI must remain understandable when color is disabled.
- The TUI must avoid requiring Nerd Fonts.
- Motion must be optional and must never be required to understand state.

## Color Capability

PAO detects terminal color capability at startup and respects explicit user configuration.

| Mode | Description | `crossterm` mapping |
| --- | --- | --- |
| `none` | No color; use glyphs, labels, and border styles | `Color::Reset` |
| `ansi16` | Basic 16-color terminal palette | `Color::Red`, `Color::DarkRed`, etc. |
| `ansi256` | 256-color indexed palette | `Color::AnsiValue(n)` |
| `truecolor` | 24-bit RGB color | `Color::Rgb { r, g, b }` |

Detection order:

1. Use `theme.color_mode` from PAO config when it is explicitly set. `never` maps to internal mode `none`.
2. Use `none` when `NO_COLOR` is set.
3. Use `none` when `CLICOLOR=0`.
4. Use the highest configured color mode when `FORCE_COLOR` is set.
5. Use `truecolor` when `COLORTERM=truecolor` or `COLORTERM=24bit`.
6. Use `ansi256` when `TERM` contains `256color`.
7. Use `none` when `TERM=dumb`.
8. Use `ansi16` for all other terminals.

Theme config shape:

```yaml
theme:
  mode: auto
  color_mode: auto
  contrast: normal
  reduced_motion: false
  ascii_borders: false
```

Allowed values:

| Setting | Values |
| --- | --- |
| `theme.mode` | `auto`, `dark`, `light` |
| `theme.color_mode` | `auto`, `never`, `ansi16`, `ansi256`, `truecolor` |
| `theme.contrast` | `normal`, `high` |
| `theme.reduced_motion` | `true`, `false` |
| `theme.ascii_borders` | `true`, `false` |

## Theme Mode

`theme.mode=auto` uses the detected terminal background when available. PAO uses `dark` when background detection is unavailable.

| Mode | Rule |
| --- | --- |
| `dark` | Keep the terminal default background and use medium-to-high luminance accents. |
| `light` | Keep the terminal default background and use darker accents. |
| `auto` | Use terminal detection and fall back to `dark`. |

PAO must not paint the entire screen with a fixed background color. Background tokens are reserved for selected rows, modals, focused panels, and meaningful highlights.

## Semantic Tokens

All UI colors must go through these semantic tokens.

| Token | Usage |
| --- | --- |
| `fg.default` | Primary text |
| `fg.muted` | Secondary text, timestamps, metadata |
| `fg.subtle` | Disabled text, placeholders |
| `fg.inverse` | Text on emphasized backgrounds |
| `border.default` | Normal panel border |
| `border.focused` | Focused panel border |
| `border.warning` | Approval, dirty repo, partial failure |
| `border.danger` | Blocked, destructive, failed command |
| `bg.default` | Terminal default background |
| `bg.surface` | Modal, popup, floating detail |
| `bg.selected` | Selected row |
| `bg.highlight` | Search match, active filter |
| `status.idle` | Idle |
| `status.running` | Running task |
| `status.success` | Completed, clean |
| `status.warning` | Needs attention |
| `status.danger` | Failure, destructive |
| `status.blocked` | Blocked run |
| `status.approval` | Waiting approval |
| `git.added` | Added line, new file |
| `git.modified` | Modified line, dirty file |
| `git.deleted` | Deleted line, removed file |
| `git.renamed` | Renamed file |
| `git.conflict` | Conflict marker, merge conflict |
| `accent.primary` | Primary action, current mode |
| `accent.secondary` | Secondary action |

## Dark Palette

Use these values when `theme.mode=dark` and truecolor is available.

| Token | RGB | 256 color | ANSI 16 fallback |
| --- | --- | --- | --- |
| `fg.default` | `#E6EDF3` | `255` | `White` |
| `fg.muted` | `#8B949E` | `245` | `DarkGrey` |
| `fg.subtle` | `#6E7681` | `242` | `DarkGrey` |
| `fg.inverse` | `#0D1117` | `232` | `Black` |
| `border.default` | `#30363D` | `238` | `DarkGrey` |
| `border.focused` | `#58A6FF` | `75` | `Blue` |
| `border.warning` | `#D29922` | `178` | `Yellow` |
| `border.danger` | `#F85149` | `203` | `Red` |
| `bg.default` | terminal default | reset | `Reset` |
| `bg.surface` | `#161B22` | `233` | `Black` |
| `bg.selected` | `#1F2A44` | `236` | `DarkBlue` |
| `bg.highlight` | `#3B2F00` | `58` | `DarkYellow` |
| `status.idle` | `#8B949E` | `245` | `DarkGrey` |
| `status.running` | `#58A6FF` | `75` | `Blue` |
| `status.success` | `#3FB950` | `35` | `Green` |
| `status.warning` | `#D29922` | `178` | `Yellow` |
| `status.danger` | `#F85149` | `203` | `Red` |
| `status.blocked` | `#FF7B72` | `210` | `Red` |
| `status.approval` | `#E3B341` | `179` | `Yellow` |
| `git.added` | `#3FB950` | `35` | `Green` |
| `git.modified` | `#D29922` | `178` | `Yellow` |
| `git.deleted` | `#F85149` | `203` | `Red` |
| `git.renamed` | `#A371F7` | `135` | `Magenta` |
| `git.conflict` | `#FF7B72` | `210` | `Red` |
| `accent.primary` | `#58A6FF` | `75` | `Blue` |
| `accent.secondary` | `#A371F7` | `135` | `Magenta` |

## Light Palette

Use these values when `theme.mode=light` and truecolor is available.

| Token | RGB | 256 color | ANSI 16 fallback |
| --- | --- | --- | --- |
| `fg.default` | `#24292F` | `235` | `Black` |
| `fg.muted` | `#57606A` | `240` | `DarkGrey` |
| `fg.subtle` | `#6E7781` | `244` | `DarkGrey` |
| `fg.inverse` | `#FFFFFF` | `255` | `White` |
| `border.default` | `#D0D7DE` | `252` | `Grey` |
| `border.focused` | `#0969DA` | `26` | `Blue` |
| `border.warning` | `#9A6700` | `94` | `DarkYellow` |
| `border.danger` | `#CF222E` | `160` | `Red` |
| `bg.default` | terminal default | reset | `Reset` |
| `bg.surface` | `#F6F8FA` | `255` | `White` |
| `bg.selected` | `#DDF4FF` | `195` | `Cyan` |
| `bg.highlight` | `#FFF8C5` | `230` | `Yellow` |
| `status.idle` | `#57606A` | `240` | `DarkGrey` |
| `status.running` | `#0969DA` | `26` | `Blue` |
| `status.success` | `#1A7F37` | `28` | `Green` |
| `status.warning` | `#9A6700` | `94` | `DarkYellow` |
| `status.danger` | `#CF222E` | `160` | `Red` |
| `status.blocked` | `#A40E26` | `124` | `DarkRed` |
| `status.approval` | `#9A6700` | `94` | `DarkYellow` |
| `git.added` | `#1A7F37` | `28` | `Green` |
| `git.modified` | `#9A6700` | `94` | `DarkYellow` |
| `git.deleted` | `#CF222E` | `160` | `Red` |
| `git.renamed` | `#8250DF` | `92` | `Magenta` |
| `git.conflict` | `#A40E26` | `124` | `DarkRed` |
| `accent.primary` | `#0969DA` | `26` | `Blue` |
| `accent.secondary` | `#8250DF` | `92` | `Magenta` |

## ANSI 16 Rules

ANSI 16 mode depends heavily on the user's terminal theme. PAO must pair colors with text labels or style modifiers.

| Token group | ANSI color | Modifier |
| --- | --- | --- |
| Body text | default foreground | none |
| Muted text | `DarkGrey` | none |
| Focused border | `Blue` | `BOLD` |
| Selected row | `Blue` | `REVERSED` or `BOLD` |
| Running | `Blue` | none |
| Success | `Green` | none |
| Warning | `Yellow` | `BOLD` |
| Danger | `Red` | `BOLD` |
| Blocked | `Red` | `BOLD` |
| Approval | `Yellow` | `BOLD` |
| Added | `Green` | none |
| Modified | `Yellow` | none |
| Deleted | `Red` | none |
| Conflict | `Red` | `BOLD` |

Use `REVERSED` only for selected rows and active tabs.

## No Color Mode

When `theme.color_mode=never`, `NO_COLOR`, or `TERM=dumb` is active, PAO must use textual prefixes and structural markers.

Status prefixes:

| Status | Prefix |
| --- | --- |
| idle | `[idle]` |
| running | `[run]` |
| success | `[ok]` |
| warning | `[warn]` |
| danger | `[fail]` |
| blocked | `[blocked]` |
| approval | `[approval]` |

Diff prefixes:

| Change | Prefix |
| --- | --- |
| added | `+` |
| modified | `~` |
| deleted | `-` |
| renamed | `>` |
| conflict | `!` |

Focus and selection rules:

- Mark the focused panel with a title marker.
- Mark the selected row with a leading `>`.
- Add `(disabled)` to disabled actions.
- Add `[destructive]` to destructive actions.

## Status Priority

Use the highest priority state when a row or panel has multiple states.

1. `blocked`
2. `danger`
3. `approval`
4. `warning`
5. `running`
6. `success`
7. `idle`

Show secondary states in the detail panel.

## Diff Rules

| Element | Token | Secondary marker |
| --- | --- | --- |
| Added line | `git.added` | `+` prefix |
| Deleted line | `git.deleted` | `-` prefix |
| Modified line | `git.modified` | `~` prefix |
| Hunk header | `accent.primary` | `@@` |
| File header | `fg.default` | bold |
| Binary file | `fg.muted` | `[binary]` |
| Renamed file | `git.renamed` | `>` prefix |
| Conflict marker | `git.conflict` | `!` prefix |
| Existing user change | `status.warning` | `[user]` |
| AI-created change | `accent.primary` | `[ai]` |
| Overlap | `status.blocked` | `[overlap]` |

Diff views should prefer foreground color and prefixes. Background tinting is reserved for high contrast mode.

## Panels and Focus

| Element | Rule |
| --- | --- |
| Normal panel | `border.default` |
| Focused panel | `border.focused` and bold title |
| Warning panel | `border.warning` |
| Danger panel | `border.danger` |
| Modal | `bg.surface`, `border.focused` |
| Approval modal | `bg.surface`, `border.warning` |
| Destructive modal | `bg.surface`, `border.danger` |

Focused panels must include a title marker:

```text
> Repo Board
```

## Terminal Compatibility

| Terminal | Rule |
| --- | --- |
| macOS Terminal | Prefer default foreground and background because ANSI 16 themes vary widely. |
| iTerm2 | Support truecolor and preserve the user's terminal background. |
| Ghostty | Validate truecolor and modern glyph rendering. |
| WezTerm | Validate truecolor, font fallback, ligatures, and wide glyph rendering. |
| Alacritty | Support truecolor and rely on config when background detection is unavailable. |
| Kitty | Support truecolor; terminal graphics protocols are out of scope for v0. |
| VS Code terminal | Validate ANSI 16 fallback because editor themes affect terminal colors. |
| tmux | Use the capability exposed inside tmux. |
| screen | Keep ANSI 16 fallback reliable. |
| SSH session | Use the remote `TERM` value. |

## Glyphs and Fonts

PAO must work without Nerd Fonts.

| Purpose | Default marker |
| --- | --- |
| Selected row | `>` |
| Expanded | `v` |
| Collapsed | `>` |
| Success | `OK` |
| Warning | `WARN` |
| Failure | `FAIL` |
| Blocked | `BLOCKED` |
| Approval | `APPROVAL` |

Unicode box drawing is allowed for the default TUI. ASCII borders must be available through `theme.ascii_borders=true`.

```text
+----------------+
| Repo Board     |
+----------------+
```

## Motion

- `theme.reduced_motion=true` replaces spinners and pulse effects with static badges.
- Running task spinners must run at no more than 4 frames per second.
- Warning, approval, and danger states use fixed badges by default.
- Progress must also be shown as text, count, or duration.

## Accessibility

- Target at least 4.5:1 contrast for normal text.
- Use labels with muted metadata when the metadata affects task understanding.
- Warning and danger states must include labels such as `[warn]`, `[fail]`, or `[blocked]`.
- Selected rows must include a leading marker.
- Focused panels must include a title marker.
- Long-running work must show status text and duration.

## Theme Config Storage

Theme config is user-level config.

```text
~/.config/pao/config.yaml
```

Theme config must not be stored in repository checkouts, `.pao/`, or session transcripts.

## Implementation Rules

- `Theme` maps semantic tokens to `ratatui::style::Style`.
- Widgets must not reference raw RGB values, ANSI indexes, or ANSI color names directly.
- Detect color capability once during TUI initialization and refresh it on config reload.
- Use `Color::Reset` to preserve the user's terminal theme.
- Limit background fills to modals, selected rows, and meaningful highlights.
- Snapshot tests must cover `none`, `ansi16`, `ansi256`, and `truecolor`.
- Primary screen snapshots must cover dark and light modes.
- Diff view snapshots must include added, modified, deleted, conflict, and overlap states.
- Approval modal snapshots must include normal, warning, and destructive states.

## Screen-Specific Rules

### Workspace Overview

- Use `accent.primary` for the top status bar.
- Use `status.warning` for dirty repo count.
- Use `status.danger` for failed run count.
- Use `status.approval` for pending approval count.
- Use `bg.selected` and a leading marker for the selected repo row.

### Active Session

- Use `fg.default` for user prompts and AI summaries.
- Use `fg.muted` for metadata.
- Show tool execution with a status token and duration.
- Show redacted values as `[redacted]`.

### Inspector

- Use `fg.default` and bold for file paths.
- Use `accent.primary` for diff hunk headers.
- Mark existing user changes with `[user]`.
- Mark AI-created changes with `[ai]`.
- Mark overlapping changes with `[overlap]` and `status.blocked`.

### Approval Modal

- Use `border.warning` for normal approvals.
- Use `border.danger` for destructive approvals.
- Show explicit actions: `Approve`, `Reject`, and `Inspect diff`.
- Show keyboard shortcuts with bracket labels.

```text
[a] Approve   [r] Reject   [d] Inspect diff   [esc] Close
```

### Command Log

- Use `fg.default` for commands.
- Use `fg.muted` for cwd.
- Use `status.success` for exit code `0`.
- Use `status.danger` for non-zero exit codes.
- Use `status.warning` for timeouts and cancellations.
- Show redacted values as `[redacted]`.

## Verification Checklist

- `NO_COLOR=1` keeps every status and diff understandable.
- `TERM=dumb` renders text fallback without panic.
- `TERM=xterm-256color` uses the 256-color palette.
- `COLORTERM=truecolor` uses the RGB palette.
- Dark mode snapshots pass.
- Light mode snapshots pass.
- tmux preserves selected row, focused border, warning, and danger distinction.
- VS Code terminal keeps ANSI 16 fallback readable.
- Approval modals communicate action risk without color.
- Diff views distinguish added, deleted, and overlapping changes without color.
- Reduced motion mode replaces spinner and pulse effects with static state.

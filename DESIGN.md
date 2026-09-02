# Audio Integrity Design System

## Direction

AudioMuse-informed operational console: calm neutral surfaces, a persistent sidebar, restrained blue actions, and high-clarity status evidence. The live scan is the dominant instrument; aggregates support it instead of competing with it.

## Tokens

- Light background `#f7f8fb`, surface `#ffffff`, text `#172033`, muted `#687386`.
- Dark background `#10141d`, surface `#181e29`, text `#edf2fa`, muted `#9ca8bb`.
- Primary blue `#2563eb`; semantic green, amber, and red are always paired with text.
- Corners are 8px for controls, 12–14px for panels, 18px only for the sign-in card.
- Typography uses the local system sans stack. Section titles stay compact; paths and validator IDs use monospace.

## Layout and behavior

- Desktop: 238px fixed sidebar and a fluid content column. Tablet collapses the sidebar to icons. Mobile moves primary navigation to a 62px bottom rail.
- The first viewport keeps scan state, actions, byte progress, current path, and outcome counts together.
- Incremental scan is primary. Full audit uses an inline confirmation because it deliberately discards cache savings.
- Long paths truncate in live state but remain complete in result rows and native title text.
- RU/EN language and light/dark theme preferences persist locally.

## Status model

- Integrity: healthy, corrupt, or validation error. Corrupt is the only content verdict that blocks Lidarr.
- Authenticity: likely lossless, likely lossy, unknown, or not applicable. This is explicitly heuristic and never presented as corruption.
- Running state uses a subtle pulsing dot and numeric byte progress; reduced-motion preferences disable animation.

## Accessibility

Semantic controls, keyboard focus rings, non-color status labels, responsive tables, minimum 38px action height, selection styling, and reduced-motion support are required. The app never relies on hover to expose the only copy of operational evidence.

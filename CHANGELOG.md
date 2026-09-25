# Changelog

## 1.0.2 - 2026-09-25

- **Fishing works on macOS and Linux.** Those terminals never tell a program that a key was let go, so the reel kept reeling after you lifted ↓ and the bar emptied in a blink. Now ↓ starts the reel and ↑ stops it there (the bottom of the fishing window says so), and steering follows your keys as they repeat. On Windows nothing changes: hold ↓ to reel.
- Console mode (`/console`) no longer sticks a key down on macOS and Linux either.

## 1.0.1 - 2026-09-25

- **`fishtanks update` works on Windows.** 1.0.0 updated itself by starting a PowerShell script, which Windows Defender blocks as a suspected trojan. It now downloads the new version and swaps itself in, with no script at all. If you have 1.0.0 on Windows, update once by running the install line again; from 1.0.1 on, `fishtanks update` does it.

## 1.0.0 - 2026-09-25

The first public release. An aquarium for your terminal, and a game under the water.

- **The aquarium.** ASCII fishes swim, eat, grow, blow bubbles and burst into zoomies on their own. `/zen` hides everything but the water.
- **Fishing** with `/fish`, a small reel-in game, and a **shop** to buy fishes, food, coffee, bait and tanks, and to sell what you catch. `/ledger` shows where the money went.
- **Tanks with their own rules**: coral, candy, a haunted graveyard, a desert with visitors at night, and tanks that are never sold, grown from what the sea gives you.
- **Twenty-odd species**, from commons to legendaries that live in only one kind of tank.
- **Mutation, fusion, life after death**, and fishes you can wire into logic, up to a calculator made of fish.
- **Always saved.** The game writes itself down as you play and when you close it. `/export` and `/import` carry it between computers.
- **One-line installs** on macOS, Linux and Windows, and a notice when a newer version is out: `fishtanks update`.

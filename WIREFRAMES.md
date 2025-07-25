## YoDA- Wireframe Design

### 1. Login/Authentication Screen

```
┌─────────────────────────────────────────────────────────────┐
│                      YoDA                                   │
│                                                              │
│                         [Logo]                               │
│                                                              │
│                    ┌─────────────────┐                      │
│      Email/Username │                 │                      │
│                    └─────────────────┘                      │
│                                                              │
│                    ┌─────────────────┐                      │
│         Password   │                 │                      │
│                    └─────────────────┘                      │
│                                                              │
│                    [ Login Button ]                          │
│                                                              │
│                  ─────────OR──────────                       │
│                                                              │
│                 [ Sign Up ] [ Forgot? ]                      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2. Dashboard - Campaign Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ [≡] YoDA                                         [🔍] [🔔] [User Avatar ▼] │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Welcome back, DM_Name!                                                      │
│                                                                              │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐ │
│  │    + New Campaign    │  │  Curse of Strahd    │  │ Lost Mine of        │ │
│  │                      │  │                     │  │ Phandelver          │ │
│  │         [+]          │  │    [Gothic Image]   │  │   [Mine Image]      │ │
│  │                      │  │                     │  │                     │ │
│  │  Create Campaign     │  │  5 Players • Active │  │  4 Players • Active │ │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘ │
│                                                                              │
│  Recent Sessions                                    Quick Actions            │
│  ┌──────────────────────────────────────────┐     ┌──────────────────────┐ │
│  │ • Session 12 - Into the Castle    2h ago │     │ [📅] Schedule Session │ │
│  │ • Session 11 - Village Terror     3d ago │     │ [👥] Invite Players   │ │
│  │ • Session 10 - The Dark Forest    1w ago │     │ [📚] Knowledge Base   │ │
│  └──────────────────────────────────────────┘     └──────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3. Main Session Interface (Core Screen)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│ [≡] Session: Castle Infiltration                     [▶ Resume] [⏸] [■ End] [Settings ⚙] │
├────────────────────┬────────────────────────────────────────┬──────────────────────────┤
│ SESSION STATUS     │              EVENT LOG                  │   AI ENHANCEMENT         │
│                    │                                          │                          │
│ 🟢 Active: 2h 15m  │ ┌──────────────────────────────────────┐│ ┌──────────────────────┐│
│                    │ │ 19:45 - Initiative Started           ││ │ Monster Stat Blocks  │││
│ Players Online (4) │ │ • Ragnar: 18                         ││ │ ┌──────────────────┐ ││
│ ┌────────────────┐ │ │ • Goblin 1: 15                       ││ │ │ Search: [_______] │ ││
│ │ 🟢 John (DM)   │ │ │ • Elara: 12                          ││ │ └──────────────────┘ ││
│ │ 🟢 Sarah       │ │ │ • Goblin 2: 8                        ││ │                      ││
│ │ 🟢 Mike        │ │ │                                      ││ │ [Goblin]             ││
│ │ 🟢 Emma        │ │ │ 19:43 - Ragnar rolled 2d20+5         ││ │ AC: 15, HP: 7        ││
│ │ 🔴 Alex        │ │ │ Result: 18 (Attack Hit!)             ││ │ STR: 8 (-1)          ││
│ └────────────────┘ │ │                                      ││ │ DEX: 14 (+2)         ││
│                    │ │ 19:42 - Emma: "I cast Healing Word   ││ │ [View Full Stats]    ││
│ Quick Tools        │ │ on Ragnar"                           ││ └──────────────────────┘│
│ ┌────────────────┐ │ │                                      ││                          │
│ │ [🎲] Dice Roll │ │ │ 19:41 - Combat Encounter Started     ││ AI Suggestions          │
│ │ [⚔️] Initiative │ │ │ Location: Castle Courtyard           ││ ┌──────────────────────┐│
│ │ [🗺️] Show Map  │ │ │ • 2 Goblins                          ││ │ "The goblins seem    ││
│ │ [📝] Add Note  │ │ │ • 1 Goblin Boss                      ││ │ nervous. Perhaps     ││
│ └────────────────┘ │ │                                      ││ │ they could be        ││
│                    │ └──────────────────────────────────────┘│ │ intimidated?"        ││
│                    │                                          │ │                      ││
│                    │ [Type message...] [Send]                 │ │ [Generate More]      ││
│                    │                                          │ └──────────────────────┘│
├────────────────────┴────────────────────────────────────────┴──────────────────────────┤
│ PLAYER & CHARACTER TRACKER                                                              │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │
│ │   Ragnar    │ │    Elara    │ │   Thorin    │ │   Lyra      │ │    Empty    │      │
│ │ Fighter L5  │ │ Wizard L5   │ │ Cleric L5   │ │ Rogue L5    │ │  [Add PC]   │      │
│ │ HP: 28/45   │ │ HP: 22/28   │ │ HP: 35/38   │ │ HP: 30/30   │ │             │      │
│ │ AC: 18      │ │ AC: 12      │ │ AC: 16      │ │ AC: 15      │ │             │      │
│ │ [Status: OK]│ │ [Concentr.] │ │ [Status: OK]│ │ [Hidden]    │ │             │      │
│ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘      │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### 4. Initiative Tracker (Modal/Overlay)

```
┌─────────────────────────────────────────────────┐
│             INITIATIVE TRACKER                   │
│                                          [X]     │
├─────────────────────────────────────────────────┤
│                                                  │
│  Current Turn: Ragnar                            │
│                                                  │
│  ┌──────────────────────────────────────────┐  │
│  │ 1. [▶] Ragnar (PC)              Init: 18 │  │
│  │ 2. [ ] Goblin 1 (NPC)          Init: 15 │  │
│  │ 3. [ ] Elara (PC)              Init: 12 │  │
│  │ 4. [ ] Thorin (PC)             Init: 10 │  │
│  │ 5. [ ] Goblin 2 (NPC)          Init: 8  │  │
│  │ 6. [ ] Lyra (PC)               Init: 5  │  │
│  └──────────────────────────────────────────┘  │
│                                                  │
│  [Previous] [Next Turn] [Reset] [Add Creature]  │
│                                                  │
└─────────────────────────────────────────────────┘
```

### 5. AI Generation Modal

```
┌─────────────────────────────────────────────────────┐
│              AI CONTENT GENERATOR                    │
│                                              [X]     │
├─────────────────────────────────────────────────────┤
│                                                      │
│  What would you like to generate?                   │
│                                                      │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐      │
│  │    NPC     │ │  Location  │ │   Quest    │      │
│  │    [👤]    │ │    [🏰]    │ │    [📜]    │      │
│  └────────────┘ └────────────┘ └────────────┘      │
│                                                      │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐      │
│  │ Encounter  │ │    Item    │ │   Puzzle   │      │
│  │    [⚔️]    │ │    [🗡️]    │ │    [🧩]    │      │
│  └────────────┘ └────────────┘ └────────────┘      │
│                                                      │
│  Context/Requirements:                               │
│  ┌──────────────────────────────────────────────┐  │
│  │                                              │  │
│  │  (Optional: Add context for better results) │  │
│  │                                              │  │
│  └──────────────────────────────────────────────┘  │
│                                                      │
│                    [Generate]                        │
│                                                      │
└─────────────────────────────────────────────────────┘
```

### 6. Knowledge Base / RAG Search

```
┌─────────────────────────────────────────────────────────────────┐
│                    KNOWLEDGE BASE                                │
│                                                          [X]     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Search: [Vampire weaknesses_____________] [🔍]                  │
│                                                                  │
│  Filter: [All Sources ▼] [All Campaigns ▼]                      │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Results (3 found):                                         │ │
│  │                                                            │ │
│  │ 📖 Monster Manual - Vampire                               │ │
│  │ "...vulnerable to sunlight, running water, and stakes     │ │
│  │ through the heart..."                                     │ │
│  │                                                            │ │
│  │ 📜 Campaign Notes - Strahd's Weaknesses                   │ │
│  │ "...the Sunsword is particularly effective..."            │ │
│  │                                                            │ │
│  │ 🎲 Custom Rule - Vampire Encounters                       │ │
│  │ "...holy symbols grant advantage on saves..."             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  [Upload Document] [Add Custom Entry]                            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 7. Battle Map View

```
┌───────────────────────────────────────────────────────────────────────────┐
│ Castle Courtyard - Battle Map                    [Grid] [Measure] [Fog] [X]│
├───────────────────────────────────────────────────────────────────────────┤
│  A   B   C   D   E   F   G   H   I   J   K   L   M   N   O   P          │
│ ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐      │
│1│▓▓▓│▓▓▓│▓▓▓│   │   │   │   │   │   │   │   │   │▓▓▓│▓▓▓│▓▓▓│▓▓▓│      │
│ ├───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┤      │
│2│▓▓▓│   │   │   │   │ G │   │   │   │   │   │   │   │   │   │▓▓▓│      │
│ ├───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┤ Tools│
│3│▓▓▓│   │ R │   │   │   │   │   │   │   │ G │   │   │   │   │▓▓▓│ ┌───┐│
│ ├───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┤ │[R]││
│4│▓▓▓│   │   │   │   │   │   │ E │   │   │   │   │   │   │   │▓▓▓│ │[E]││
│ ├───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┤ │[T]││
│5│   │   │   │ T │   │   │   │   │   │   │   │   │   │   │   │   │ │[L]││
│ ├───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┤ │[G]││
│6│   │   │   │   │   │   │   │   │ L*│   │   │   │   │   │   │   │ └───┘│
│ ├───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┤      │
│7│   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │ Move │
│ ├───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┤ Draw │
│8│▓▓▓│▓▓▓│▓▓▓│▓▓▓│▓▓▓│   │   │   │   │   │   │▓▓▓│▓▓▓│▓▓▓│▓▓▓│▓▓▓│ Erase│
│ └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘      │
│ Legend: R=Ragnar E=Elara T=Thorin L=Lyra(*hidden) G=Goblin ▓=Wall       │
└───────────────────────────────────────────────────────────────────────────┘
```

### 8. Mobile Responsive View (Player Companion)

```
┌─────────────────────┐
│ ≡  D&D Assistant  🔔 │
├─────────────────────┤
│                     │
│   Ragnar            │
│   Fighter Level 5   │
│                     │
│  ┌─────────────────┐│
│  │   HP: 28/45     ││
│  │   ━━━━━━░░░░░   ││
│  │                 ││
│  │   AC: 18        ││
│  │   Speed: 30ft   ││
│  └─────────────────┘│
│                     │
│  Quick Actions      │
│  ┌────────┬────────┐│
│  │  Roll  │  Rest  ││
│  │   🎲   │   🏕️   ││
│  └────────┴────────┘│
│                     │
│  Stats              │
│  STR: 18 (+4) ▼     │
│  DEX: 14 (+2) ▼     │
│  CON: 16 (+3) ▼     │
│                     │
│  [View Full Sheet]  │
│                     │
├─────────────────────┤
│ [Session][Character]│
└─────────────────────┘
```

### User Flow Diagram

```
                    ┌─────────┐
                    │  Login  │
                    └────┬────┘
                         │
                    ┌────▼────┐
                    │Dashboard│
                    └────┬────┘
                         │
            ┌────────────┼────────────┐
            │            │            │
       ┌────▼────┐  ┌────▼────┐  ┌───▼────┐
       │Campaign │  │ Create  │  │ Join   │
       │  List   │  │Campaign │  │Session │
       └────┬────┘  └─────────┘  └───┬────┘
            │                         │
       ┌────▼────┐              ┌────▼────┐
       │ Session │              │ Active  │
       │  Setup  │              │ Session │◄─────┐
       └────┬────┘              └────┬────┘      │
            │                        │           │
            └────────────┬───────────┘           │
                         │                       │
                    ┌────▼────┐                  │
                    │  Main   │                  │
                    │ Session │──────────────────┘
                    │Interface│
                    └─────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    ┌────▼────┐    ┌────▼────┐    ┌────▼────┐
    │   AI    │    │ Battle  │    │Knowledge│
    │Generate │    │   Map   │    │  Base   │
    └─────────┘    └─────────┘    └─────────┘
```

## Key Design Principles

1. **Three-Panel Layout**: Main session screen uses a three-panel design for maximum information density
   - Left: Session status and quick tools
   - Center: Event log and main interaction area
   - Right: AI assistance and contextual help

2. **Persistent Character Tracker**: Bottom bar shows all PCs with critical stats always visible

3. **Real-time Updates**: Event log and status indicators update in real-time via WebSocket

4. **Context-Aware AI**: AI panel changes based on current activity (combat, exploration, roleplay)

5. **Mobile-First Player View**: Simplified interface for players to manage their characters on phones/tablets

6. **Quick Access Tools**: Most common DM actions are one-click away

7. **Modular Overlays**: Complex features (maps, detailed generators) open as overlays to maintain context

This wireframe design prioritizes:
- Quick access to frequently used tools
- Real-time collaboration features
- AI integration at every step
- Clean, scannable information hierarchy
- Responsive design for various devices
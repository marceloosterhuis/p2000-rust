# P2000 Rust TUI

A terminal user interface (TUI) application for reading and decoding Dutch P2000 emergency services messages.

## What is P2000?

P2000 is a **digital alerting network** used by Dutch emergency services (fire brigades, ambulances, police, rescue teams, and others) to alert personnel of incidents. The system broadcasts unencrypted messages using the FLEX protocol on 169.650 MHz.

### Key Features of P2000:

- **Unencrypted**: Messages are publicly readable
- **Digital**: Replaced the old analog system
- **Real-time**: Instant dispatch of emergency units
- **Scalable**: Supports alerting many units simultaneously

### Message Fields:

Each P2000 message contains:
- **Priority**: P1-P3 for fire services, A0-A2/B for ambulances
- **Incident Code**: Unit designation (e.g., BDH-07, BRT-03)
- **Incident Type**: What happened (Accident, Fire, Water overflow, etc.)
- **Location**: Where the incident occurred
- **Capcodes**: Device IDs of individual units being alerted
- **Reference Numbers**: For tracking purposes

### Priority Levels:

**Fire Brigade (Brandweer):**
- P1: Highest priority - use lights and sirens
- P2: Urgent but lower priority
- P3: Non-urgent (mostly unused)

**Ambulances:**
- A0: Life-threatening emergency requiring immediate intervention
- A1: Urgent (within 15 minutes) with lights and sirens
- A2: Urgent but without lights/sirens
- B: Planned transport

## Application Features

### TUI Interface:
- **Message List**: Scrollable list of all parsed messages with priority color coding
- **Detail View**: Shows complete information about the selected message
- **Search**: Find messages by content, location, or priority (press 's' to toggle)
- **Navigation**: Arrow keys and Page Up/Down for scrolling

### Supported Input Sources:
- **File**: `cargo run -- ./path/to/data.txt`
- **Stdin**: `cat messages.txt | cargo run`

### Message Parsing:

The parser automatically extracts:
- Timestamp
- Protocol type (FLEX)
- Radio address
- Frequency
- Capcodes (unit device IDs)
- Message type (ALN for alert)
- Message content with embedded:
  - Priority level (P1-P3, A0-A2, B)
  - Incident code (e.g., BDH-07)
  - Location
  - Additional details

## Building

```bash
cargo build --release
```

## Running

### With example data:
```bash
cargo run --release -- ./data/p2000-1.txt
```

### With stdin:
```bash
cat your_file.txt | cargo run
```

## Controls

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate through messages |
| PageUp/Down | Jump 10 messages |
| s | Toggle search mode |
| (in search) Backspace | Delete character |
| (in search) Enter | Exit search |
| q / Esc | Quit application |

## Project Structure

```
src/
├── main.rs       # Application entry point
├── parser.rs     # P2000 message parser
├── reader.rs     # File and stdin reader
└── tui.rs        # Terminal UI implementation
data/
└── p2000-1.txt   # Example P2000 message data
```

## Example Data

The `data/p2000-1.txt` file contains real-world P2000 messages from 2026-01-01, showing various incident types across the Netherlands.

## Dependencies

- **ratatui**: Terminal UI framework
- **crossterm**: Terminal manipulation
- **tokio**: Async runtime
- **regex**: Pattern matching for message parsing
- **chrono**: DateTime handling
- **thiserror**: Error handling
- **anyhow**: Error context

## Future Enhancements

- Capcode database lookup to identify units by name
- Filtering by incident type or region
- Message archival and statistics
- Real-time P2000 receiver integration
- Incident type recognition and categorization
- Map integration showing incident locations

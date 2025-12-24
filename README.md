# trexanh

> trexanh is a GitHub contribution graph TUI, written in 🦀 Rust :))

<img src="assets/demo.jpg">

## Installation & Usage

### 1. Download the binary

Download the latest release of `trexanh` for your platform

### 2. Make it executable

```bash
chmod +x trexanh
```

### 3. Get a GitHub fine-grained personal access token

1. Go to [GitHub Settings > Developer settings > Personal access tokens > Fine-grained tokens]
2. Click "Generate new token"
3. Give your token a name and set an expiration date
4. Under "Repository access", select **Public repositories**
5. Click "Generate token" and copy it

### 4. Run

#### Basic usage

```bash
./trexanh
```
and input your GitHub username and token

#### With flags

```bash
./trexanh --cached
```
this flag will instantly display the cached contributions while fetching new data for the next run in the background

```bash
./trexanh --watch <SECONDS>
```
this flag will continuously fetch and update contributions (**DO NOT** use very short intervals to avoid hammering the GitHub API)

```bash
./trexanh --reset
```
this flag will prompt you to update your stored username and token

> You can optionally input a different username to see their contributions

## Roadmap

- [x] Add argument to get other username's contribution graph
- [x] Add async cache with background fetch (fork) to display cached data instantly and refresh in the background (cached mode)
- [x] Add watch mode to refresh graph at intervals
- [x] Add TUI for username and token input

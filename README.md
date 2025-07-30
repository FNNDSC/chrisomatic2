# chrisomatic

[![Test](https://github.com/FNNDSC/chrisomatic2/actions/workflows/test.yml/badge.svg)](https://github.com/FNNDSC/chrisomatic2/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/FNNDSC/chrisomatic2/graph/badge.svg?token=LN6ZBB25T2)](https://codecov.io/gh/FNNDSC/chrisomatic2)

Declarative setup of _ChRIS_ backend (_CUBE_) for automatic testing and demos.
https://chrisproject.org/

## Usage

`chrisomatic` is both a command-line program and a web app component.

### Web App

`chrisomatic` is included in the [ChRIS_ui](https://github.com/FNNDSC/ChRIS_ui).
Open ChRIS_ui and log in as an admin user.

- If no feeds are present, a component on the dashboard (home page) will offer
  "first time setup" functionality which uses `chrisomatic` to populate _CUBE_
  with sample data.
- You can access this component at any time by clicking the "chrisomatic" tab
  in the left sidebar.

### Installation

The command-line interface (CLI) of `chrisomatic` can be installed many ways:

- **Direct download** from https://github.com/FNNDSC/chrisomatic2/releases/latest
- **Compile from source**: `cargo install chrisomatic`
- **cargo-binstall**: `cargo binstall chrisomatic`
  <sup style="font-size: 50%">
    <a href="https://github.com/cargo-bins/cargo-binstall">
      (What is <code>cargo-binstall</code>?)
    </a>
  </sup>
- **npm, pnpm, yarn, or bun**: e.g. `npm install -g @fnndsc/chrisomatic` or
  `bun install --global @fnndsc/chrisomatic`
- **pip**: `pip install chrisomatic`
- **[Nix](https://nixos.org/) flakes**: figure it out yourself

### Design

_chrisomatic_ is needlessly overengineered for the love of programming.
Its design is motivated by these questions:

- We want to show a progress bar. How can we estimate the total amount of
  needed "work" before execution, and the current "progress" during
  execution?
- Some API requests have dependencies (e.g. user must exist before getting
  their auth token) whereas others can run concurrently (e.g. creation of
  user "alice" can happen concurrently with creation of user "bobby"). How
  can we maximize concurrency, without doing it explicitly (i.e. use
  [`select!`](https://docs.rs/futures/latest/futures/macro.select.html)
  or equivalent no more than once throughout the entire codebase).

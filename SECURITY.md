# Security Policy

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, email **renan.pimentel@gmail.com** with:

- a description of the issue and its impact,
- steps to reproduce (a proof of concept if possible),
- any suggested remediation.

You can expect an initial response within a few days. Please give us a reasonable window to
investigate and ship a fix before any public disclosure.

## Scope & threat model

jobRabbit drives a **real, logged-in browser** (Google Chrome via the Claude in Chrome extension)
and spawns the `claude` CLI on your machine. Keep in mind:

- It runs **locally**: there is no jobRabbit server, account, or telemetry. Your profile, answers
  and settings live under `~/.local/share/jobrabbit/`.
- It acts with **your** browser session and credentials. Treat the machine running it as you would
  any tool with access to your logged-in accounts.
- Secrets (if any) are stored via the OS keyring (Secret Service), never in the repo.

Reports we especially care about: anything that could exfiltrate local data, inject untrusted
input into the agent prompt in a harmful way, or cause the tool to act outside the user's intent.

## Supported versions

This is an early-stage project; security fixes target the latest `main`.

# System Monitor Skill

Checks system resources: RAM, disk space, and uptime.

## What It Does

Runs `free -h`, `df -h`, and `uptime` to give a quick snapshot of system
health.  Useful for verifying that the shell (hardware) has enough capacity
before running resource-intensive operations.

## Example Output

```
              total        used        free      shared  buff/cache   available
Mem:           7.7Gi       2.1Gi       3.4Gi       256Mi       2.2Gi       5.1Gi
Swap:          2.0Gi          0B       2.0Gi

Filesystem      Size  Used Avail Use% Mounted on
/dev/sda2       234G   45G  178G  21% /

 10:23:45 up 42 days,  3:17,  2 users,  load average: 0.12, 0.08, 0.05
```

## Edge Cases

- On macOS, `free` is not available; the reflex should be re-compiled to use
  `vm_stat` and `sysctl` instead.
- Docker containers may report host-level metrics depending on configuration.
- If the system is in CRITICAL resource mode, this reflex still runs (it's
  lightweight and helps the PID controller make decisions).

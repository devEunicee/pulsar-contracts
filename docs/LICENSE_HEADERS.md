# License Headers

All source files in the Pulsar project must include an SPDX license identifier header.

## Requirement

Every Rust source file (`.rs`) must start with the following line:

```rust
// SPDX-License-Identifier: MIT
```

This line must be the **first line** of the file, before any other code, comments, or attributes.

## Why License Headers?

1. **Compliance** — Many open-source compliance tools (REUSE, SPDX, etc.) require explicit license declarations
2. **Clarity** — Makes it immediately clear what license applies to each file
3. **Legal** — Provides explicit copyright and license information for legal purposes
4. **Automation** — Enables automated license scanning and compliance checking

## Examples

### Correct

```rust
// SPDX-License-Identifier: MIT

#![no_std]

extern crate alloc;

mod error;
```

### Incorrect

```rust
// This file is part of Pulsar
// SPDX-License-Identifier: MIT

#![no_std]
```

(License header must be the first line)

## Adding Headers to Existing Files

If you're adding a new Rust file, include the header at the top:

```bash
# Create new file with header
echo "// SPDX-License-Identifier: MIT" > src/new_module.rs
echo "" >> src/new_module.rs
# Then add your code
```

Or manually add it to the top of the file before any other content.

## CI Enforcement

The CI pipeline includes an automated check (`scripts/check-license-headers.sh`) that verifies all Rust files have the required header. This check runs on every pull request and will fail if any file is missing the header.

### Running the Check Locally

```bash
bash scripts/check-license-headers.sh
```

If the check fails, it will list all files missing headers and show the required format.

## License Information

- **License**: MIT
- **SPDX Identifier**: MIT
- **License File**: [LICENSE](../LICENSE)

For more information about the MIT license, see https://opensource.org/licenses/MIT

## References

- [SPDX License List](https://spdx.org/licenses/)
- [REUSE Software](https://reuse.software/)
- [SPDX Specification](https://spdx.github.io/spdx-spec/)

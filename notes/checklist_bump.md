### 1. Bump the version in `Cargo.toml`

Edit manually, or use `cargo set-version` (from [`cargo-edit`](https://github.com/killercup/cargo-edit)):

```bash
cargo install cargo-edit     # only once
cargo set-version 0.2.0      # replace with your new version
```

This updates `[package].version` in `Cargo.toml` and syncs `Cargo.lock`.

---

### 2. Commit the change

```bash
git add Cargo.toml Cargo.lock
git commit -m "Bump version to 0.2.0"
```

**HINT**

git config --global alias.ac 'commit -am'

Then I can run

git ac "bump version to 0.5.0"


---

### 3. Tag the release

```bash
git tag v0.2.0
```

---

### 4. Push commit and tag to GitHub

```bash
git push origin main
git push origin v0.2.0
```

---

### 5. (Optional) Publish locally

If you want to double-check before CI does it:

```bash
cargo publish --dry-run   # sanity check
cargo publish             # actually publish
```
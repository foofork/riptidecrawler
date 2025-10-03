# Publishing RipTide CLI to npm

## Prerequisites

1. **npm Account**
   - Create account at https://www.npmjs.com/signup
   - Verify email address

2. **npm Authentication**
   ```bash
   npm login
   # Enter username, password, and email
   ```

3. **Generate npm Token** (for CI/CD)
   - Go to https://www.npmjs.com/settings/tokens
   - Create new token (Automation or Publish)
   - Add to GitHub Secrets as `NPM_TOKEN`

## Manual Publishing

### 1. Prepare Package

```bash
cd cli

# Install dependencies
npm install

# Run tests
npm test

# Run linter
npm run lint

# Verify package contents
npm pack --dry-run
```

### 2. Update Version

```bash
# Update version in package.json
npm version patch  # 1.0.0 -> 1.0.1
npm version minor  # 1.0.0 -> 1.1.0
npm version major  # 1.0.0 -> 2.0.0

# Or manually edit package.json
```

### 3. Publish to npm

```bash
# Publish
npm publish --access public

# Verify
npm view @riptide/cli
```

### 4. Test Installation

```bash
# In new directory
npm install -g @riptide/cli

# Test it works
riptide --version
riptide health
```

## Automated Publishing (GitHub Actions)

### Setup

1. **Add npm Token to GitHub Secrets**
   - Go to repository Settings → Secrets → Actions
   - Add secret: `NPM_TOKEN` = your npm token

2. **Tag Release**
   ```bash
   # Create and push tag
   git tag cli-v1.0.0
   git push origin cli-v1.0.0
   ```

3. **GitHub Actions automatically**:
   - Runs tests
   - Builds package
   - Publishes to npm
   - Creates GitHub release

### Manual Trigger

1. Go to Actions tab on GitHub
2. Select "Publish CLI to npm"
3. Click "Run workflow"

## Version Naming

Follow semantic versioning:

- **Major** (X.0.0): Breaking changes
- **Minor** (1.X.0): New features, backward compatible
- **Patch** (1.0.X): Bug fixes

```bash
# Examples
npm version patch  # Bug fix: 1.0.0 -> 1.0.1
npm version minor  # New feature: 1.0.0 -> 1.1.0
npm version major  # Breaking change: 1.0.0 -> 2.0.0
```

## Pre-release Versions

```bash
# Alpha
npm version prerelease --preid=alpha  # 1.0.0 -> 1.0.1-alpha.0

# Beta
npm version prerelease --preid=beta   # 1.0.0 -> 1.0.1-beta.0

# Publish with tag
npm publish --tag beta
```

## Publishing Checklist

Before publishing:

- [ ] All tests pass
- [ ] Linter passes
- [ ] Version bumped in package.json
- [ ] CHANGELOG.md updated
- [ ] README.md up to date
- [ ] No debug/console.log statements
- [ ] Dependencies updated
- [ ] Build tested locally
- [ ] Examples work
- [ ] Documentation reviewed

## Verification After Publishing

```bash
# Check npm registry
npm view @riptide/cli

# Check versions
npm view @riptide/cli versions

# Test installation
npm install -g @riptide/cli@latest

# Verify CLI works
riptide --version
riptide examples
```

## Unpublishing (Emergency Only)

⚠️ **Warning**: Can only unpublish within 72 hours

```bash
# Unpublish specific version
npm unpublish @riptide/cli@1.0.0

# Unpublish entire package (use with extreme caution!)
npm unpublish @riptide/cli --force
```

## Beta Testing

1. Publish beta version:
   ```bash
   npm version prerelease --preid=beta
   npm publish --tag beta
   ```

2. Users install beta:
   ```bash
   npm install -g @riptide/cli@beta
   ```

3. Promote to stable:
   ```bash
   npm dist-tag add @riptide/cli@1.1.0-beta.0 latest
   ```

## Troubleshooting

### Package Name Conflict

```bash
# Check if name is available
npm view @riptide/cli

# If taken, update package.json with new name
"name": "@your-org/riptide-cli"
```

### Authentication Errors

```bash
# Re-login to npm
npm logout
npm login

# Verify authentication
npm whoami
```

### Version Exists

```bash
# Error: Cannot publish over existing version

# Bump version
npm version patch

# Then publish
npm publish
```

### Access Denied

```bash
# Make sure you have access
npm access grant read-write your-org:team @riptide/cli

# Or publish as public
npm publish --access public
```

## Best Practices

1. **Test Thoroughly**
   - Run full test suite
   - Test in clean environment
   - Test installation from npm

2. **Version Carefully**
   - Follow semantic versioning
   - Update CHANGELOG
   - Tag releases in git

3. **Document Changes**
   - Update README
   - Write clear CHANGELOG entries
   - Include migration guide for breaking changes

4. **Use CI/CD**
   - Automate testing
   - Automate publishing
   - Prevent manual errors

5. **Monitor Downloads**
   - Check npm stats
   - Monitor issues
   - Collect user feedback

## Support

- npm Documentation: https://docs.npmjs.com/
- Semantic Versioning: https://semver.org/
- GitHub Actions: https://docs.github.com/en/actions

## Links

- npm Package: https://www.npmjs.com/package/@riptide/cli
- GitHub Repository: https://github.com/your-org/riptide-api
- Issues: https://github.com/your-org/riptide-api/issues

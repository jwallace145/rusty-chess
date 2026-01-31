module.exports = {
  branches: ['main'],
  repositoryUrl: 'https://x-access-token:${GH_TOKEN}@github.com/jwallace145/rusty-chess',
  plugins: [
    '@semantic-release/commit-analyzer',
    '@semantic-release/release-notes-generator',
    [
      '@semantic-release/changelog',
      {
        changelogFile: 'CHANGELOG.md',
      },
    ],
    [
      '@semantic-release/git',
      {
        assets: ['CHANGELOG.md'], // TODO: add Cargo.toml if you want it synced automatically
        message:
          'chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}',
      },
    ],
    '@semantic-release/github',
  ],
};

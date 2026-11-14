module.exports = {
  extends: ["@commitlint/config-conventional"],
  rules: {
    // Cursor/IDE commit UIs often emit unwrapped body bullets; keep conventional
    // types/subjects but do not reject long body lines.
    "body-max-line-length": [0],
    "type-enum": [
      2,
      "always",
      [
        "feat",
        "fix",
        "docs",
        "style",
        "refactor",
        "perf",
        "test",
        "build",
        "ci",
        "chore",
        "revert",
      ],
    ],
  },
};

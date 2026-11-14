/// Marks sandbox directories with NSFileManager `isExcludedFromBackupKey`.
///
/// Android excludes the same data with `android:allowBackup="false"`.
void excludeSandboxesFromBackup(
  List<String> directories,
  void Function(String path) markExcluded,
) {
  for (final directory in directories) {
    markExcluded(directory);
  }
}

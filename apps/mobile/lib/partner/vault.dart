import 'dart:io';

/// Ciphertext-only partner cache. Plaintext and the epoch key stay in memory.
class PartnerVault {
  PartnerVault(this.root);

  final Directory root;
  String? phaseLabel;
  String? fingerprint;

  void show({required String phase, required String trackerFingerprint}) {
    phaseLabel = phase;
    fingerprint = trackerFingerprint;
  }

  void lock() {
    phaseLabel = null;
    fingerprint = null;
  }

  Future<void> persistCiphertext(List<int> ciphertext) async {
    final file = File('${root.path}/cache.bin');
    await file.writeAsBytes(ciphertext, flush: true);
  }

  Future<void> wipe() async {
    lock();
    final file = File('${root.path}/cache.bin');
    if (await file.exists()) {
      await file.delete();
    }
  }

  /// A later fetch writes ciphertext only when the relay still has a body.
  Future<void> applyFetch(List<int>? ciphertext) async {
    if (ciphertext == null) {
      return;
    }
    await persistCiphertext(ciphertext);
  }
}

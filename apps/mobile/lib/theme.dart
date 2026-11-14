import 'package:flutter/material.dart';

/// Forest green seed for the Material 3 color system.
const Color forestGreen = Color(0xFF228B22);

/// Plain Material 3 theme. Light and dark both come from [forestGreen].
ThemeData ouroborosTheme({Brightness brightness = Brightness.light}) {
  return ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(
      seedColor: forestGreen,
      brightness: brightness,
    ),
  );
}

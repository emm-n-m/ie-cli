// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.pvrz;

/** Supported Pvrz compression types. */
public enum CompressionType {
  /** Most appropriate compression type is chosen automatically. */
  AUTO("Autodetect"),
  /** Enforce DXT1 compression type. */
  DXT1("DXT1 (opaque)"),
  /** Enforce DXT5 compression type. */
  DXT5("DXT5 (alpha-blended)"),
  ;

  private static final String COMPRESSION_HELP =
      "\"DXT1\" provides the highest compression ratio. It supports only 1 bit alpha\n"
    + "(no or full transparency) and is the preferred type for TIS and MOS resources.\n\n"
    + "\"DXT5\" provides an average compression ratio. It features interpolated\n"
    + "alpha-blending and is the preferred type for BAM resources.\n\n"
    + "\"Autodetect\" selects the most appropriate compression type based on the input data.";

  private final String label;

  CompressionType(String label) {
    this.label = label;
  }

  @Override
  public String toString() {
    return label;
  }

  /** Returns a help description about the available compression types. */
  public static String getHelp() {
    return COMPRESSION_HELP;
  }
}
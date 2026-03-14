// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.spl.generator;

/**
 * Interface for {@code enum} types that can be used by the {@link InputTracker} class.
 * Defines numeric bounds and default values.
 */
public interface InputBounds {
  /** Returns the lower bound for the given control. */
  int getMinValue();

  /** Returns the upper bound for the given control. */
  int getMaxValue();

  /** Returns the default value for the given control. */
  int getDefaultValue();

  /**
   * A scaler that should be applied to the value. Returns {@code 0} by default.
   * <p>
   * Applied as <tt>value &times; 10<sup>-scale</sup></tt>
   * </p>
   */
  default int getScaleValue() {
    return 0;
  }
}

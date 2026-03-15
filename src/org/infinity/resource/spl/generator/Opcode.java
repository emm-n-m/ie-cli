// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.spl.generator;

import java.util.Objects;

import org.infinity.resource.effects.BaseOpcode;

/**
 * Represents a single opcode with description text.
 */
public class Opcode extends Number implements Comparable<Integer> {
  public static final String UNKNOWN_NAME = "Unknown";

  private final int opcode;

  /**
   * Constructs a newly allocated {@code OpcodeItem} object that represents the specified opcode value.
   *
   * @param opcode Opcode to be represented by this {@code OpcodeItem} instance.
   */
  public Opcode(int opcode) {
    super();
    this.opcode = opcode;
  }

  /**
   * Returns the descriptive name of the opcode.
   *
   * @return Opcode name as {@code String}.
   */
  public String getName() {
    final String[] names = BaseOpcode.getEffectNames();
    if (names != null && opcode >= 0 && opcode < names.length) {
      return names[opcode];
    }
    return UNKNOWN_NAME;
  }

  @Override
  public int intValue() {
    return opcode;
  }

  @Override
  public long longValue() {
    return opcode;
  }

  @Override
  public float floatValue() {
    return opcode;
  }

  @Override
  public double doubleValue() {
    return opcode;
  }

  @Override
  public int compareTo(Integer o) {
    return Integer.compare(opcode, o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(opcode);
  }

  @Override
  public boolean equals(Object obj) {
    if (this == obj) {
      return true;
    }
    if (obj == null) {
      return false;
    }
    if (getClass() != obj.getClass()) {
      return false;
    }
    Opcode other = (Opcode)obj;
    return opcode == other.opcode;
  }

  @Override
  public String toString() {
    return getName() + " (" + intValue() + ')';
  }
}

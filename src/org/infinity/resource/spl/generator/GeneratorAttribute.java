// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.spl.generator;

import java.io.Serializable;
import java.util.Objects;

/**
 * Specialized structure for holding base and increment values for a single spell attribute.
 */
public class GeneratorAttribute implements Serializable {
  /** Version number for serialization. */
  private static final long SERIAL_VERSION = 1L;
  private static final long serialVersionUID = SERIAL_VERSION | ((long)GeneratorAttribute.class.getSimpleName().hashCode() << 32);

  private final String label;

  private int baseValue;
  private double incValue;
  private boolean enabled;

  /** Initializes the attribute with 0 for both values and default labels for {@link #toString()}. */
  public GeneratorAttribute() {
    this(0, 0.0, null, true);
  }

  /**
   * Initializes the attribute with 0 for both values and default labels for {@link #toString()}.
   *
   * @param enabled Whether the attribute is enabled (optional property).
   */
  public GeneratorAttribute(boolean enabled) {
    this(0, 0.0, null, enabled);
  }

  /**
   * Initializes the attribute with the specified parameters for both values and default labels for {@link #toString()}.
   *
   * @param base  Base value.
   * @param inc   Increment value.
   * @param label Label that identifies this attribute (only used for diagnostic output).
   */
  public GeneratorAttribute(int base, double inc) {
    this(base, inc, null, true);
  }

  /**
   * Initializes the attribute with the specified parameters for both values and default labels for {@link #toString()}.
   *
   * @param base    Base value.
   * @param inc     Increment value.
   * @param label   Label that identifies this attribute (only used for diagnostic output).
   * @param enabled Whether the attribute is enabled (optional property).
   */
  public GeneratorAttribute(int base, double inc, boolean enable) {
    this(base, inc, null, enable);
  }

  /**
   * Initializes the attribute with the specified parameters for values and labels.
   *
   * @param base  Base value.
   * @param inc   Increment value.
   * @param label Label that identifies this attribute (only used for diagnostic output).
   */
  public GeneratorAttribute(int base, double inc, String label) {
    this(base, inc, label, true);
  }

  /**
   * Initializes the attribute with the specified parameters for values and labels as well as the enabled state.
   *
   * @param base    Base value.
   * @param inc     Increment value.
   * @param label   Label that identifies this attribute (only used for diagnostic output).
   * @param enabled Whether the attribute is enabled (optional property).
   */
  public GeneratorAttribute(int base, double inc, String label, boolean enabled) {
    this.baseValue = base;
    this.incValue = inc;
    this.label = (label != null) ? label : "Attribute";
    this.enabled = enabled;
  }

  /** Returns the base value of the attribute. */
  public int getBase() {
    return baseValue;
  }

  /** Sets a new base value for the attribute. Returns a reference to this object. */
  public GeneratorAttribute setBase(int newValue) {
    baseValue = newValue;
    return this;
  }

  /** Returns the increment value of the attribute. */
  public double getIncrement() {
    return incValue;
  }

  /** Sets a new increment value of the attribute. Returns a reference to this object. */
  public GeneratorAttribute setIncrement(double newValue) {
    incValue = newValue;
    return this;
  }

  /** (Optional property) Returns whether the attribute is enabled. */
  public boolean isEnabled() {
    return enabled;
  }

  /** (Optional property) Sets the enabled state of the attribute. Returns a reference to this object. */
  public GeneratorAttribute setEnabled(boolean newValue) {
    enabled = newValue;
    return this;
  }

  /**
   * Assigns the values from the "attr" parameter to this object. Labels are unaffected. Returns a reference to this
   * object.
   */
  public GeneratorAttribute setAttribute(GeneratorAttribute attr) {
    Objects.requireNonNull(attr);
    baseValue = attr.baseValue;
    incValue = attr.incValue;
    enabled = attr.enabled;
    return this;
  }

  /**
   * Convenience function that calculates a scaled attribute value.
   *
   * @param incrementFactor Factor that is multiplied with the increment value of the attribute.
   * @return Scaled attribute value.
   */
  public int getScaledValue(int incrementFactor) {
    return getScaledValue(incrementFactor, 1);
  }

  /**
   * Convenience function that calculates a scaled attribute value.
   *
   * @param incrementFactor Factor that is multiplied with the increment value of the attribute.
   * @param globalFactor    Factor that is multiplied with the scaled attribute value.
   * @return Scaled attribute value.
   */
  public int getScaledValue(int incrementFactor, int globalFactor) {
    int retVal = baseValue;
    if (incValue < 0) {
      retVal += (int)Math.ceil(incValue * Math.max(0, incrementFactor));
    } else {
      retVal += (int)Math.floor(incValue * Math.max(0, incrementFactor));
    }
    retVal *= Math.max(1, globalFactor);
    return retVal;
  }

  @Override
  public int hashCode() {
    return Objects.hash(baseValue, incValue, enabled);
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
    GeneratorAttribute other = (GeneratorAttribute)obj;
    return baseValue == other.baseValue && Double.doubleToLongBits(incValue) == Double.doubleToLongBits(other.incValue)
        && enabled == other.enabled;
  }

  @Override
  public String toString() {
    final StringBuilder sb = new StringBuilder();
    sb.append(label).append("=(Base: ").append(baseValue).append(", Increment: ").append(incValue).append(')');
    return sb.toString();
  }
}

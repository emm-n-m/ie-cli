// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.spl.generator;

import java.io.Serializable;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

import org.infinity.resource.effects.BaseOpcode;
import org.infinity.util.Misc;

/**
 * Storage for effect-specific configurations.
 */
public class EffectConfig implements Serializable, Comparable<EffectConfig> {
  /** Version number for serialization. */
  private static final long SERIAL_VERSION = 1L;
  private static final long serialVersionUID = SERIAL_VERSION | ((long)EffectConfig.class.getSimpleName().hashCode() << 32);

  /** Enum with supported parameter configuration modes. */
  public enum Mode {
    /** Parameter is handled as a full 32-bit {@code DWORD} value. */
    DWORD(4, 1, "as 1 DWORD", "DWORD"),
    /** Parameter is handled as two separate 16-bit {@code WORD} values. */
    WORD(2, 2, "as 2 WORDs", "WORD"),
    /** Parameter is handled as four separate 8-bit {@code BYTE} values. */
    BYTE(1, 4, "as 4 BYTEs", "BYTE"),;

    private final int size;
    private final int count;
    private final String desc;
    private final String label;

    Mode(int size, int count, String desc, String label) {
      this.size = size;
      this.count = count;
      this.desc = desc;
      this.label = label;
    }

    /** Returns the size per item, in bytes. */
    public int getItemSize() {
      return size;
    }

    /** Returns the number of individual items that can be stored by this mode. */
    public int getItemCount() {
      return count;
    }

    /** Description string for this mode for use as list item text. */
    public String getDescription() {
      return desc;
    }

    /** Short label for a individual item. */
    public String getLabel() {
      return label;
    }

    @Override
    public String toString() {
      return getDescription();
    }

    /** Returns an array of strings with the {@link Mode} descriptions. */
    public static String[] getModeStrings() {
      final Mode[] values = values();
      final String[] retVal = new String[values.length];
      for (int i = 0; i < values.length; i++) {
        retVal[i] = values[i].getDescription();
      }
      return retVal;
    }
  }

  private final List<GeneratorAttribute> param1Items = new ArrayList<>(Mode.BYTE.getItemCount());
  private final List<GeneratorAttribute> param2Items = new ArrayList<>(Mode.BYTE.getItemCount());
  private final List<GeneratorAttribute> specialItems = new ArrayList<>(Mode.BYTE.getItemCount());

  private int opcode;
  private Mode param1Mode;
  private Mode param2Mode;
  private Mode specialMode;

  /** Initializes the configuration with default values. */
  public EffectConfig() {
    setParameter1Mode(Mode.DWORD);
    setParameter2Mode(Mode.DWORD);
    setSpecialMode(Mode.DWORD);
  }

  /**
   * Initializes the configuration with the specified parameters.
   *
   * @param opcode     Effect opcode.
   * @param param1     Full parameter 1 attributes.
   * @param param1Low  Low word parameter 1 attributes.
   * @param param1High High word parameter 1 attributes.
   * @param param2     Full parameter 2 attributes.
   * @param param2Low  Low word parameter 2 attributes.
   * @param param2High High word parameter 2 attributes.
   * @param special    Special attributes.
   */
  public EffectConfig(int opcode, Mode param1Mode, Collection<GeneratorAttribute> param1Items, Mode param2Mode,
      Collection<GeneratorAttribute> param2Items, Mode specialMode, Collection<GeneratorAttribute> specialItems) {
    setOpcode(opcode);

    setParameter1Mode(Misc.orDefault(param1Mode, Mode.DWORD));
    initParameter(getParameter1Mode(), param1Items, this.param1Items);

    setParameter2Mode(Misc.orDefault(param2Mode, Mode.DWORD));
    initParameter(getParameter2Mode(), param2Items, this.param2Items);

    setSpecialMode(Misc.orDefault(specialMode, Mode.DWORD));
    initParameter(getSpecialMode(), specialItems, this.specialItems);
  }

  /** Returns the effect opcode. */
  public int getOpcode() {
    return opcode;
  }

  /** Sets a new opcode value. */
  public void setOpcode(int newOpcode) {
    opcode = newOpcode;
  }

  /** Returns the configuration {@link Mode} used for "Parameter 1". */
  public Mode getParameter1Mode() {
    return param1Mode;
  }

  /**
   * Sets a new configuration {@link Mode} for "Parameter 1". Available list items are adjusted accordingly.
   *
   * @param newMode The new {@code Mode}.
   */
  public void setParameter1Mode(Mode newMode) {
    if (newMode != null && newMode != param1Mode) {
      param1Mode = newMode;
      while (param1Items.size() < param1Mode.getItemCount()) {
        param1Items.add(new GeneratorAttribute(false));
      }
      while (param1Items.size() > param1Mode.getItemCount()) {
        param1Items.remove(param1Items.size() - 1);
      }
    }
  }

  /**
   * Provides access to the "Parameter 1" item at the specified index.
   *
   * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid "Parameter 1" items can be
   *                                     determined by {@link Mode#getItemCount()}.
   * @see #getParameter1Mode()
   */
  public GeneratorAttribute getParameter1(int index) {
    return param1Items.get(index);
  }

  /** Provides access to all "Parameter 1" items at one. */
  public List<GeneratorAttribute> getParameter1Items() {
    return Collections.unmodifiableList(param1Items);
  }

  /** Returns the configuration {@link Mode} used for "Parameter 2". */
  public Mode getParameter2Mode() {
    return param2Mode;
  }

  /**
   * Sets a new configuration {@link Mode} for "Parameter 2". Available list items are adjusted accordingly.
   *
   * @param newMode The new {@code Mode}.
   */
  public void setParameter2Mode(Mode newMode) {
    if (newMode != null && newMode != param2Mode) {
      param2Mode = newMode;
      while (param2Items.size() < param2Mode.getItemCount()) {
        param2Items.add(new GeneratorAttribute(false));
      }
      while (param2Items.size() > param2Mode.getItemCount()) {
        param2Items.remove(param2Items.size() - 1);
      }
    }
  }

  /**
   * Provides access to the "Parameter 2" item at the specified index.
   *
   * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid "Parameter 2" items can be
   *                                     determined by {@link Mode#getItemCount()}.
   * @see #getParameter2Mode()
   */
  public GeneratorAttribute getParameter2(int index) {
    return param2Items.get(index);
  }

  /** Provides access to all "Parameter 2" items at one. */
  public List<GeneratorAttribute> getParameter2Items() {
    return Collections.unmodifiableList(param2Items);
  }

  /** Returns the configuration {@link Mode} used for the "Special" parameter. */
  public Mode getSpecialMode() {
    return specialMode;
  }

  /**
   * Sets a new configuration {@link Mode} for the "Special" parameter. Available list items are adjusted accordingly.
   *
   * @param newMode The new {@code Mode}.
   */
  public void setSpecialMode(Mode newMode) {
    if (newMode != null && newMode != specialMode) {
      specialMode = newMode;
      while (specialItems.size() < specialMode.getItemCount()) {
        specialItems.add(new GeneratorAttribute(false));
      }
      while (specialItems.size() > specialMode.getItemCount()) {
        specialItems.remove(specialItems.size() - 1);
      }
    }
  }

  /**
   * Provides access to the "Special" parameter item at the specified index.
   *
   * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid "Special" parameter items
   *                                     can be determined by {@link Mode#getItemCount()}.
   * @see #getSpecialMode()
   */
  public GeneratorAttribute getSpecial(int index) {
    return specialItems.get(index);
  }

  /** Provides access to all "Special" parameter items at one. */
  public List<GeneratorAttribute> getSpecialItems() {
    return Collections.unmodifiableList(specialItems);
  }

  @Override
  public int compareTo(EffectConfig o) {
    return (o != null) ? Integer.compare(opcode, o.opcode) : 1;
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
    EffectConfig other = (EffectConfig)obj;
    return opcode == other.opcode;
  }

  @Override
  public String toString() {
    final String[] names = BaseOpcode.getEffectNames();
    final StringBuilder sb = new StringBuilder();
    sb.append("Effect: ");
    if (opcode < 0 || opcode >= names.length) {
      sb.append(Opcode.UNKNOWN_NAME);
    } else {
      sb.append(names[opcode]);
    }
    sb.append(" (").append(opcode).append(')');

    int count = 0;
    String s = getParameterString("Parameter 1", param1Mode, param1Items);
    if (!s.isEmpty()) {
      sb.append(count == 0 ? ':' : ';').append(' ');
      sb.append(s);
      count++;
    }

    s = getParameterString("Parameter 2", param2Mode, param2Items);
    if (!s.isEmpty()) {
      sb.append(count == 0 ? ':' : ';').append(' ');
      sb.append(s);
      count++;
    }

    s = getParameterString("Special", specialMode, specialItems);
    if (!s.isEmpty()) {
      sb.append(count == 0 ? ':' : ';').append(' ');
      sb.append(s);
      count++;
    }

    return sb.toString();
  }

  /** Returns a descriptive substring for the specified parameter. */
  private String getParameterString(String label, Mode mode, List<GeneratorAttribute> list) {
    boolean isEnabled = false;
    for (int i = 0; !isEnabled && i < mode.getItemCount(); i++) {
      isEnabled = list.get(i).isEnabled();
    }

    // scheme: "<label> [MODE]=(<idx>: <substring>; <idx>: <substring>; ...)"
    if (isEnabled) {
      final StringBuilder sb = new StringBuilder();
      sb.append(label).append(' ').append('[').append(mode.getLabel()).append("]=(");
      for (int i = 0, j = 0; i < mode.getItemCount(); i++) {
        if (list.get(i).isEnabled()) {
          if (j > 0) {
            sb.append(';').append(' ');
          }
          if (mode.getItemCount() > 1) {
            sb.append(i + 1).append(':').append(' ');
          }
          sb.append(list.get(i).toString());
          j++;
        }
      }
      sb.append(')');
      return sb.toString();
    }

    return "";
  }

  /**
   * Initializes a single parameter definition.
   *
   * @param mode     The current parameter mode.
   * @param srcItems Collection of items to assign to the parameter.
   * @param dstItems List of configuration items.
   */
  private void initParameter(Mode mode, Collection<GeneratorAttribute> srcItems, List<GeneratorAttribute> dstItems) {
    if (mode != null && srcItems != null && dstItems != null) {
      final int count = mode.getItemCount();
      int i = 0;
      for (final GeneratorAttribute srcItem : srcItems) {
        if (i >= count) {
          break;
        }
        if (srcItem != null) {
          dstItems.get(i).setAttribute(srcItem);
        }
        i++;
      }
    }
  }

  // -------------------------- INNER CLASSES --------------------------

  /***
   * Helper class for managing data of a single parameter.
   */
  public static class Parameter {
    private final List<GeneratorAttribute> itemList;
    private final Mode mode;

    private int incrementFactor;
    private int globalFactor;

    /**
     * Initializes the parameter.
     *
     * @param mode  Configuration {@link Mode} of the parameter.
     * @param items Collection of parameter items. Number must be compatible with the {@code mode} parameter.
     */
    public Parameter(Mode mode, Collection<GeneratorAttribute> items) {
      this(mode, items, 0, 1);
    }

    /**
     * Initializes the parameter.
     *
     * @param mode            Configuration {@link Mode} of the parameter.
     * @param items           Collection of parameter items. Number must be compatible with the {@code mode} parameter.
     * @param incrementFactor Factor that is multiplied with the incrementation value.
     * @param globalFactor    Factor that is multiplied with the calculated parameter value.
     */
    public Parameter(Mode mode, Collection<GeneratorAttribute> items, int incrementFactor, int globalFactor) {
      this.mode = Objects.requireNonNull(mode);
      this.itemList = new ArrayList<>(Objects.requireNonNull(items).size());
      int count = this.mode.getItemCount();
      for (final GeneratorAttribute attr : items) {
        if (count > 0) {
          final GeneratorAttribute ga = new GeneratorAttribute();
          ga.setAttribute(attr);
          itemList.add(ga);
        }
        count--;
      }
      if (count < 0) {
        throw new IllegalArgumentException("Mismatching number of parameter items.");
      }
      setIncrementFactor(incrementFactor);
      setGlobalFactor(globalFactor);
    }

    /** Returns the configuration {@link Mode}. */
    public Mode getMode() {
      return mode;
    }

    /** Returns the parameter item list. */
    public List<GeneratorAttribute> getItems() {
      return Collections.unmodifiableList(itemList);
    }

    /** Returns the factor that is multiplied with the incrementation value of the parameter item. (Default: 0) */
    public int getIncrementFactor() {
      return incrementFactor;
    }

    /** Sets the factor that is multiplied with the incrementation value of the parameter item. */
    public Parameter setIncrementFactor(int factor) {
      incrementFactor = Math.max(0, factor);
      return this;
    }

    /** Returns the global factor that is multiplied with the calculated parameter value. (Default: 1) */
    public int getGlobalFactor() {
      return globalFactor;
    }

    /** Sets the global factor that is multiplied with the calculated parameter value. */
    public Parameter setGlobalFactor(int factor) {
      globalFactor = Math.max(0, factor);
      return this;
    }

    /**
     * Returns a {@link Byte} array from the scaled parameter items.
     *
     * @return {@code Byte} array with the base value content. Unused bytes are set to {@code null}.
     */
    public Byte[] getBytes() {
      final Byte[] retVal = new Byte[4];
      switch (mode) {
        case BYTE:
          for (int i = 0; i < 4; ++i) {
            if (itemList.get(i).isEnabled()) {
              retVal[i] = (byte)itemList.get(i).getScaledValue(getIncrementFactor(), getGlobalFactor());
            }
          }
          break;
        case WORD:
          if (itemList.get(0).isEnabled()) {
            short value = (short)itemList.get(0).getScaledValue(getIncrementFactor(), getGlobalFactor());
            retVal[0] = (byte)(value & 0xff);
            retVal[1] = (byte)((value >>> 8) & 0xff);
          }
          if (itemList.get(1).isEnabled()) {
            short value = (short)itemList.get(1).getScaledValue(getIncrementFactor(), getGlobalFactor());
            retVal[2] = (byte)(value & 0xff);
            retVal[3] = (byte)((value >>> 8) & 0xff);
          }
          break;
        default:
          if (itemList.get(0).isEnabled()) {
            int value = itemList.get(0).getScaledValue(getIncrementFactor(), getGlobalFactor());
            retVal[0] = (byte)(value & 0xff);
            retVal[1] = (byte)((value >>> 8) & 0xff);
            retVal[2] = (byte)((value >>> 16) & 0xff);
            retVal[3] = (byte)((value >>> 24) & 0xff);
          }
      }
      return retVal;
    }
  }
}

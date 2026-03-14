// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.spl.generator;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InvalidClassException;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.io.Serializable;
import java.util.Collection;
import java.util.Collections;
import java.util.HashMap;
import java.util.Iterator;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;
import java.util.zip.Deflater;
import java.util.zip.DeflaterOutputStream;
import java.util.zip.InflaterInputStream;

/**
 * Provides read-only access to the parameters ofthe ability generation feature.
 */
public class GeneratorConfig implements Serializable {
  /** Version number for serialization. */
  private static final long SERIAL_VERSION = 1L;
  private static final long serialVersionUID = SERIAL_VERSION | ((long)GeneratorConfig.class.getSimpleName().hashCode() << 32);

  /** Empty set of opcodes. */
  public static final Set<Integer> EMPTY_SET = Collections.unmodifiableSet(new TreeSet<>());

  private final GeneratorAttribute range = new GeneratorAttribute(0, 0.0, "Range");
  private final GeneratorAttribute castingSpeed = new GeneratorAttribute(0, 0.0, "Casting speed");
  private final GeneratorAttribute duration = new GeneratorAttribute(0, 0.0, "Duration");
  private final GeneratorAttribute diceCount = new GeneratorAttribute(0, 0.0, "Dice count");
  private final GeneratorAttribute diceSize = new GeneratorAttribute(0, 0.0, "Dice size");
  private final GeneratorAttribute saveBonus = new GeneratorAttribute(0, 0.0, "Save bonus");

  private final Map<Integer, EffectConfig> effectConfigs = new HashMap<>();
  private final Set<Integer> durationBlackList = new TreeSet<>();

  private int levelsPerAbil;
  private int maxLevel;
  private int durationThreshold;
  private boolean skipUnneeded;

  /**
   * Static method that loads a configuration from disk and returns it as a {@link GeneratorConfig} instance.
   *
   * @param configFile Input {@link File} path to load configuration data from.
   * @return a new {@link GeneratorConfig} instance if successful.
   * @throws Exception if the load operation could not be performed.
   */
  public static GeneratorConfig load(File configFile) throws Exception {
    if (configFile == null) {
      throw new NullPointerException("configFile is null");
    }

    try (final ObjectInputStream ois = new ObjectInputStream(new InflaterInputStream(new FileInputStream(configFile)))) {
      final Object object = ois.readObject();
      if (!(object instanceof GeneratorConfig)) {
        throw new InvalidClassException("File contains unknown datatypes.");
      }
      final GeneratorConfig config = (GeneratorConfig)object;
      return config;
    } catch (InvalidClassException e) {
      throw new Exception("Incompatible file type.", e);
    }
  }

  /** Initializes an empty {@code GeneratorConfig} instance. */
  public GeneratorConfig() {
  }

  /**
   * Save this {@link GeneratorConfig} instance to disk.
   *
   * @param configFile Output {@link File} path to save configuration data.
   * @throws Exception if the save operation could not be performed.
   */
  public void save(File configFile) throws Exception {
    if (configFile == null) {
      throw new NullPointerException("configFile is null");
    }

    final Deflater def = new Deflater(Deflater.BEST_COMPRESSION);
    try (final FileOutputStream fos = new FileOutputStream(configFile);
        final DeflaterOutputStream dos = new DeflaterOutputStream(fos, def);
        final ObjectOutputStream oos = new ObjectOutputStream(dos)) {
      oos.writeObject(this);
      dos.finish();
    }
  }

  /** Returns the number of levels a single ability structure should cover. */
  public int getLevelsPerAbility() {
    return levelsPerAbil;
  }

  /** Sets the number of levels a single ability structure should cover. */
  protected void setLevelsPerAbility(int newValue) {
    levelsPerAbil = newValue;
  }

  /** Returns the highest level an ability structure should be created for. */
  public int getMaxLevel() {
    return maxLevel;
  }

  /** Sets the highest level an ability structure should be created for. */
  protected void setMaxLevel(int newValue) {
    maxLevel = newValue;
  }

  /** Returns whether the "Range" attribute should be modified. */
  public boolean isRangeEnabled() {
    return range.isEnabled();
  }

  /** Sets whether the "Range" attribute should be modified. */
  public void setRangeEnabled(boolean newValue) {
    range.setEnabled(newValue);
  }

  /** Returns the initial "Range" value for level 1. */
  public int getRangeBase() {
    return range.getBase();
  }

  /** Sets the initial "Range" value for level 1. */
  protected void setRangeBase(int newValue) {
    range.setBase(newValue);
  }

  /**
   * Returns the incremental "Range" value for every new ability structure.
   * Fractional values can be used to span multiple abilities with the same value.
   */
  public double getRangeIncrement() {
    return range.getIncrement();
  }

  /** Sets the incremental "Range" value for every new ability structure. */
  protected void setRangeIncrement(double newValue) {
    range.setIncrement(newValue);
  }

  /** Returns whether the "Casting Speed" attribute should be modified. */
  public boolean isCastingSpeedEnabled() {
    return castingSpeed.isEnabled();
  }

  /** Sets whether the "Casting Speed" attribute should be modified. */
  public void setCastingSpeedEnabled(boolean newValue) {
    castingSpeed.setEnabled(newValue);
  }

  /** Returns the initial "Casting Speed" value for level 1. */
  public int getCastingSpeedBase() {
    return castingSpeed.getBase();
  }

  /** Sets the initial "Casting Speed" value for level 1. */
  protected void setCastingSpeedBase(int newValue) {
    castingSpeed.setBase(newValue);
  }

  /**
   * Returns the incremental "Casting Speed" value for every new ability structure.
   * Fractional values can be used to span multiple abilities with the same value.
   */
  public double getCastingSpeedIncrement() {
    return castingSpeed.getIncrement();
  }

  /** Sets the incremental "Casting Speed" value for every new ability structure. */
  protected void setCastingSpeedIncrement(double newValue) {
    castingSpeed.setIncrement(newValue);
  }

  /** Returns whether the "Duration" attribute should be modified. */
  public boolean isDurationEnabled() {
    return duration.isEnabled();
  }

  /** Sets whether the "Duration" attribute should be modified. */
  public void setDurationEnabled(boolean newValue) {
    duration.setEnabled(newValue);
  }

  /** Returns the initial "Duration" value for level 1. */
  public int getDurationBase() {
    return duration.getBase();
  }

  /** Sets the initial "Duration" value for level 1. */
  protected void setDurationBase(int newValue) {
    duration.setBase(newValue);
  }

  /**
   * Returns the incremental "Duration" value for every new ability structure.
   * Fractional values can be used to span multiple abilities with the same value.
   */
  public double getDurationIncrement() {
    return duration.getIncrement();
  }

  /** Sets the incremental "Duration" value for every new ability structure. */
  protected void setDurationIncrement(double newValue) {
    duration.setIncrement(newValue);
  }

  /** Returns the threshold for durations that should be preserved. */
  public int getDurationThreshold() {
    return durationThreshold;
  }

  /** Sets the threshold for durations that should be preserved. */
  protected void setDurationThreshold(int newValue) {
    durationThreshold = newValue;
  }

  /** Returns a set of opcodes where duration should be preserved. */
  public Set<Integer> getDurationBlackList() {
    return durationBlackList;
  }

  /** Assigns a set of opcodes where duration should be preserved. Old values are discarded. */
  protected void setDurationBlackList(Set<Integer> set) {
    durationBlackList.clear();

    if (set != null) {
      durationBlackList.addAll(set);
    }
  }

  /** Convenience method that returns a set of opcodes where duration must be set even for instant timing modes. */
  public Set<Integer> getForcedDurationList() {
    return GeneratorFrame.getForcedDurationList();
  }

  /** Returns whether the "Dice Count" attribute should be modified. */
  public boolean isDiceCountEnabled() {
    return diceCount.isEnabled();
  }

  /** Sets whether the "Dice Count" attribute should be modified. */
  public void setDiceCountEnabled(boolean newValue) {
    diceCount.setEnabled(newValue);
  }

  /** Returns the initial "Dice Count" value for level 1. */
  public int getDiceCountBase() {
    return diceCount.getBase();
  }

  /** Sets the initial "Dice Count" value for level 1. */
  protected void setDiceCountBase(int newValue) {
    diceCount.setBase(newValue);
  }

  /**
   * Returns the incremental "Dice Count" value for every new ability structure.
   * Fractional values can be used to span multiple abilities with the same value.
   */
  public double getDiceCountIncrement() {
    return diceCount.getIncrement();
  }

  /** Sets the incremental "Dice Count" value for every new ability structure. */
  protected void setDiceCountIncrement(double newValue) {
    diceCount.setIncrement(newValue);
  }

  /** Returns whether the "Dice Sice" attribute should be modified. */
  public boolean isDiceSizeEnabled() {
    return diceSize.isEnabled();
  }

  /** Sets whether the "Dice Size" attribute should be modified. */
  public void setDiceSizeEnabled(boolean newValue) {
    diceSize.setEnabled(newValue);
  }

  /** Returns the initial "Dice Size" value for level 1. */
  public int getDiceSizeBase() {
    return diceSize.getBase();
  }

  /** Sets the initial "Dice Size" value for level 1. */
  protected void setDiceSizeBase(int newValue) {
    diceSize.setBase(newValue);
  }

  /**
   * Returns the incremental "Dice Size" value for every new ability structure.
   * Fractional values can be used to span multiple abilities with the same value.
   */
  public double getDiceSizeIncrement() {
    return diceSize.getIncrement();
  }

  /** Sets the incremental "Dice Size" value for every new ability structure. */
  protected void setDiceSizeIncrement(double newValue) {
    diceSize.setIncrement(newValue);
  }

  /** Returns whether the "Save Bonus" attribute should be modified. */
  public boolean isSaveBonusEnabled() {
    return saveBonus.isEnabled();
  }

  /** Sets whether the "Save Bonus" attribute should be modified. */
  public void setSaveBonusEnabled(boolean newValue) {
    saveBonus.setEnabled(newValue);
  }

  /** Returns the initial "Save Bonus" value for level 1. */
  public int getSaveBonusBase() {
    return saveBonus.getBase();
  }

  /** Sets the initial "Save Bonus" value for level 1. */
  protected void setSaveBonusBase(int newValue) {
    saveBonus.setBase(newValue);
  }

  /**
   * Returns the incremental "Save Bonus" value for every new ability structure.
   * Fractional values can be used to span multiple abilities with the same value.
   */
  public double getSaveBonusIncrement() {
    return saveBonus.getIncrement();
  }

  /** Sets the incremental "Save Bonus" value for every new ability structure. */
  protected void setSaveBonusIncrement(double newValue) {
    saveBonus.setIncrement(newValue);
  }

  /** Returns whether ability structures for levels that don't meet the spell level requirement should be skipped. */
  public boolean isSkipUnneeded() {
    return skipUnneeded;
  }

  /** Sets whether ability structures for levels that don't meet the spell level requirement should be skipped. */
  protected void setSkipUnneeded(boolean newValue) {
    skipUnneeded = newValue;
  }

  /** Returns a effect-specific configurations, mapped to opcodes, for use by the ability generation. */
  public Map<Integer, EffectConfig> getEffectConfigs() {
    return effectConfigs;
  }

  /** Assigns a set of effect-specific configurations for use by the ability generation. Old content is discarded. */
  protected void setEffectConfigs(Collection<EffectConfig> effectConfigs) {
    this.effectConfigs.clear();
    if (effectConfigs != null) {
      for (final Iterator<EffectConfig> iter = effectConfigs.iterator(); iter.hasNext(); ) {
        final EffectConfig config = iter.next();
        this.effectConfigs.put(config.getOpcode(), config);
      }
    }
  }

  @Override
  public String toString() {
    StringBuilder builder = new StringBuilder();
    builder.append("GeneratorConfig [levelsPerAbil=").append(levelsPerAbil)
        .append(", maxLevel=").append(maxLevel)
        .append(", range=").append(range)
        .append(", castingSpeed=").append(castingSpeed)
        .append(", duration=").append(duration)
        .append(", durationThreshold=").append(durationThreshold)
        .append(", durationBlackList=").append(durationBlackList)
        .append(", diceCount=").append(diceCount)
        .append(", diceSize=").append(diceSize)
        .append(", saveBonus=").append(saveBonus)
        .append(", effectConfigs=").append(effectConfigs)
        .append(", skipUnneeded=").append(skipUnneeded)
        .append(']');
    return builder.toString();
  }
}

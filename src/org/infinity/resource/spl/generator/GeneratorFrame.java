// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.spl.generator;

import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.io.File;
import java.text.NumberFormat;
import java.text.ParseException;
import java.util.Arrays;
import java.util.Collections;
import java.util.EnumMap;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Locale;
import java.util.Objects;
import java.util.Set;
import java.util.TreeSet;

import javax.swing.JButton;
import javax.swing.JCheckBox;
import javax.swing.JComponent;
import javax.swing.JFileChooser;
import javax.swing.JFormattedTextField;
import javax.swing.JLabel;
import javax.swing.JMenuItem;
import javax.swing.JOptionPane;
import javax.swing.JPanel;
import javax.swing.SwingUtilities;
import javax.swing.filechooser.FileNameExtensionFilter;

import org.infinity.gui.ButtonPopupMenu;
import org.infinity.gui.ViewerUtil;
import org.infinity.icon.Icons;
import org.infinity.resource.Profile;
import org.infinity.resource.spl.SplResource;
import org.infinity.resource.spl.generator.InputTracker.ValidationException;
import org.tinylog.Logger;

/**
 * Panel that provides customization options for the SPL ability generation feature.
 * Use the nested class {@link GeneratorDialog} to embed it in a dialog.
 */
public class GeneratorFrame extends JPanel implements ActionListener {
  /**
   * List of effects which should preserve original duration for non-instant timing modes.
   * Used internally by {@code performAbilityGeneration()}.
   */
  private static final EnumMap<Profile.Engine, Set<Integer>> EFFECTS_PRESERVE_TIMING = new EnumMap<>(Profile.Engine.class);
  /**
   * List of effects which should always preserve their timing even for instant timing modes (because of engine bugs, ...).
   * Used internally by {@code performAbilityGeneration()}.
   */
  private static final Set<Integer> EFFECTS_FORCE_TIMING =
      Collections.unmodifiableSet(new TreeSet<>(Arrays.asList(new Integer[] { 78, 98, })));

  static {
    EFFECTS_PRESERVE_TIMING.put(Profile.Engine.EE,
        Collections.unmodifiableSet(new TreeSet<>(Arrays.asList(new Integer[] { 50, 215, 318, 324, }))));
    EFFECTS_PRESERVE_TIMING.put(Profile.Engine.Unknown, EFFECTS_PRESERVE_TIMING.get(Profile.Engine.EE));
    EFFECTS_PRESERVE_TIMING.put(Profile.Engine.BG1, EFFECTS_PRESERVE_TIMING.get(Profile.Engine.EE));
    EFFECTS_PRESERVE_TIMING.put(Profile.Engine.BG2, EFFECTS_PRESERVE_TIMING.get(Profile.Engine.EE));

    EFFECTS_PRESERVE_TIMING.put(Profile.Engine.IWD,
        Collections.unmodifiableSet(new TreeSet<>(Arrays.asList(new Integer[] { 50, }))));

    EFFECTS_PRESERVE_TIMING.put(Profile.Engine.IWD2,
        Collections.unmodifiableSet(new TreeSet<>(Arrays.asList(new Integer[] { 50, }))));

    EFFECTS_PRESERVE_TIMING.put(Profile.Engine.PST,
        Collections.unmodifiableSet(new TreeSet<>(Arrays.asList(new Integer[] { 50, 187, 188, 189, 190, 191, }))));
  }

  private static GeneratorFrame instance;

  private final InputTracker<InputControl> tracker = new InputTracker<>(InputControl.class);
  private final HashMap<JCheckBox, Set<JComponent>> linkedControls = new HashMap<>();

  private GeneratorConfig config;
  private GeneratorDialog dialog;
  private SplResource struct;
  private JFormattedTextField tfLevelsPerAbil, tfMaxLevels;
  private JFormattedTextField tfRangeBase, tfRangeInc;
  private JFormattedTextField tfCastingSpeedBase, tfCastingSpeedInc;
  private JFormattedTextField tfDurationBase, tfDurationInc, tfDurationThreshold;
  private JFormattedTextField tfDiceCountBase, tfDiceCountInc;
  private JFormattedTextField tfDiceSizeBase, tfDiceSizeInc;
  private JFormattedTextField tfSaveBonusBase, tfSaveBonusInc;
  private JCheckBox cbRange, cbCastingSpeed, cbDuration, cbDiceCount, cbDiceSize, cbSaveBonus;
  private JCheckBox cbDurationBlackList, cbSkipUnneeded;
  private JButton bDurationBlackList, bGenerate, bCancel, bReset, bAdvanced;
  private ButtonPopupMenu bpmProfiles;
  private JMenuItem miImport, miExport;
  private BlackListDialog blackListDialog;
  private EffectsListDialog effectsListDialog;

  /**
   * Returns a set of opcode numbers which should preserve their independent duration.
   *
   * @param defaultsOnly Indicates whether to return the default set ({@code true}) or the customized set.
   * @return A set of opcodes.
   */
  protected static Set<Integer> getDurationBlackList(boolean defaultsOnly) {
    if (defaultsOnly ||
        instance == null ||
        (instance.cbDurationBlackList.isSelected() && instance.blackListDialog == null)) {
      return EFFECTS_PRESERVE_TIMING.getOrDefault(Profile.getEngine(), EFFECTS_PRESERVE_TIMING.get(Profile.Engine.BG2));
    } else if (instance.cbDurationBlackList.isSelected()) {
      return instance.blackListDialog.getConfig();
    } else {
      return GeneratorConfig.EMPTY_SET;
    }
  }

  /**
   * Returns a set of opcode numbers that should always provide a duration regardless of timing mode.
   *
   * @return A set of opcode.
   */
  protected static Set<Integer> getForcedDurationList() {
    return EFFECTS_FORCE_TIMING;
  }

  /** Returns the {@link GeneratorFrame} singleton instance. */
  protected static GeneratorFrame getInstance() {
    if (instance == null) {
      instance = new GeneratorFrame();
    }
    return instance;
  }

  /** Removes the static instance of this class to clear all input data. */
  protected static void clearInstance() {
    instance = null;
  }

  private GeneratorFrame() {
    super();
    init();
    config = createConfig();
  }

  /** Returns the currently associated {@link SplResource} structure. */
  protected SplResource getStructure() {
    return struct;
  }

  /** Assigns a new {@link SplResource} structure to the panel. */
  protected void setStructure(SplResource struct) {
    this.struct = Objects.requireNonNull(struct);
  }

  /** Returns the currently associated parent dialog.*/
  protected GeneratorDialog getDialog() {
    return dialog;
  }

  /** Assigns a new parent dialog to the panel. */
  protected void setDialog(GeneratorDialog dialog) {
    this.dialog = Objects.requireNonNull(dialog);
  }

  /** Returns a {@link GeneratorConfig} instance. */
  protected GeneratorConfig getConfig() {
    return config;
  }

  /** Applies the specified config to the UI controls. */
  protected void applyConfig(GeneratorConfig config) {
    if (config == null) {
      return;
    }

    cbDurationBlackList.setSelected(!config.getDurationBlackList().isEmpty());
    if (blackListDialog == null) {
      blackListDialog = new BlackListDialog(getDialog(), config.getDurationBlackList());
    } else {
      blackListDialog.applyConfig(config.getDurationBlackList());
    }

    cbSkipUnneeded.setSelected(config.isSkipUnneeded());

    for (final InputControl key : tracker.getControls()) {
      Number value = null;
      switch (key) {
        case LEVELS_PER_ABIL:
          value = config.getLevelsPerAbility();
          break;
        case MAX_LEVELS:
          value = config.getMaxLevel();
          break;
        case RANGE_BASE:
          value = config.getRangeBase();
          break;
        case RANGE_INC:
          cbRange.setSelected(config.isRangeEnabled());
          onChecked(cbRange);
          value = config.getRangeIncrement();
          break;
        case CASTING_SPEED_BASE:
          value = config.getCastingSpeedBase();
          break;
        case CASTING_SPEED_INC:
          cbCastingSpeed.setSelected(config.isCastingSpeedEnabled());
          onChecked(cbCastingSpeed);
          value = config.getCastingSpeedIncrement();
          break;
        case DURATION_BASE:
          value = config.getDurationBase();
          break;
        case DURATION_INC:
          cbDuration.setSelected(config.isDurationEnabled());
          onChecked(cbDuration);
          value = config.getDurationIncrement();
          break;
        case DURATION_THRESHOLD:
          value = config.getDurationThreshold();
          break;
        case DICE_COUNT_BASE:
          cbDiceCount.setSelected(config.isDiceCountEnabled());
          onChecked(cbDiceCount);
          value = config.getDiceCountBase();
          break;
        case DICE_COUNT_INC:
          value = config.getDiceCountIncrement();
          break;
        case DICE_SIZE_BASE:
          cbDiceSize.setSelected(config.isDiceSizeEnabled());
          onChecked(cbDiceSize);
          value = config.getDiceSizeBase();
          break;
        case DICE_SIZE_INC:
          value = config.getDiceSizeIncrement();
          break;
        case SAVE_BONUS_BASE:
          cbSaveBonus.setSelected(config.isSaveBonusEnabled());
          onChecked(cbSaveBonus);
          value = config.getSaveBonusBase();
          break;
        case SAVE_BONUS_INC:
          value = config.getSaveBonusIncrement();
          break;
        default:
      }

      final JFormattedTextField tf = tracker.get(key);
      if (value instanceof Double) {
        tf.setText(Double.toString(value.doubleValue()));
      } else if (value instanceof Integer) {
        tf.setText(Integer.toString(value.intValue()));
      }
    }

    if (effectsListDialog == null) {
      effectsListDialog = new EffectsListDialog(getDialog());
    }
    effectsListDialog.applyConfig(config.getEffectConfigs().values());
  }

  /**
   * Creates and returns a fully initialized {@link GeneratorConfig} structure from the current state of UI controls.
   */
  protected GeneratorConfig createConfig() {
    final GeneratorConfig config = new GeneratorConfig();
    for (final InputControl key : tracker.getControls()) {
      switch (key) {
        case LEVELS_PER_ABIL:
          config.setLevelsPerAbility(tracker.getValue(key).intValue());
          break;
        case MAX_LEVELS:
          config.setMaxLevel(tracker.getValue(key).intValue());
          break;
        case RANGE_BASE:
          config.setRangeEnabled(cbRange.isSelected());
          config.setRangeBase(tracker.getValue(key).intValue());
          break;
        case RANGE_INC:
          config.setRangeIncrement(tracker.getValue(key).doubleValue());
          break;
        case CASTING_SPEED_BASE:
          config.setCastingSpeedEnabled(cbCastingSpeed.isSelected());
          config.setCastingSpeedBase(tracker.getValue(key).intValue());
          break;
        case CASTING_SPEED_INC:
          config.setCastingSpeedIncrement(tracker.getValue(key).doubleValue());
          break;
        case DURATION_BASE:
          config.setDurationEnabled(cbDuration.isSelected());
          config.setDurationBase(tracker.getValue(key).intValue());
          break;
        case DURATION_INC:
          config.setDurationIncrement(tracker.getValue(key).doubleValue());
          break;
        case DURATION_THRESHOLD:
          config.setDurationThreshold(tracker.getValue(key).intValue());
          break;
        case DICE_COUNT_BASE:
          config.setDiceCountEnabled(cbDiceCount.isSelected());
          config.setDiceCountBase(tracker.getValue(key).intValue());
          break;
        case DICE_COUNT_INC:
          config.setDiceCountIncrement(tracker.getValue(key).doubleValue());
          break;
        case DICE_SIZE_BASE:
          config.setDiceSizeEnabled(cbDiceSize.isSelected());
          config.setDiceSizeBase(tracker.getValue(key).intValue());
          break;
        case DICE_SIZE_INC:
          config.setDiceSizeIncrement(tracker.getValue(key).doubleValue());
          break;
        case SAVE_BONUS_BASE:
          config.setSaveBonusEnabled(cbSaveBonus.isSelected());
          config.setSaveBonusBase(tracker.getValue(key).intValue());
          break;
        case SAVE_BONUS_INC:
          config.setSaveBonusIncrement(tracker.getValue(key).doubleValue());
          break;
        default:
      }
    }

    if (cbDurationBlackList.isSelected()) {
      config.setDurationBlackList(getDurationBlackList(false));
    } else {
      config.setDurationBlackList(null);
    }

    config.setSkipUnneeded(cbSkipUnneeded.isSelected());

    if (effectsListDialog != null) {
      config.setEffectConfigs(effectsListDialog.getConfig());
    } else {
      config.setEffectConfigs(null);
    }

    return config;
  }

  /** Resets the content of all input fields to their respective default value. */
  protected void reset() {
    tracker.resetInput();
    cbDurationBlackList.setSelected(false);
    onDurationBlackListCheck(cbDuration.isSelected() && cbDurationBlackList.isSelected());
    cbSkipUnneeded.setSelected(true);
    blackListDialog = null;
    effectsListDialog = null;
    config = null;
  }

  /**
   * Checks whether current configuration is valid in the current context and optionally shows a summary of the
   * requested operation.
   */
  private boolean validateInput(boolean showSummary) {
    try {
      tracker.validateInput(true);
    } catch (ValidationException e) {
      JOptionPane.showMessageDialog(this, e.getMessage(), "Error", JOptionPane.ERROR_MESSAGE);
      if (e.getControl() != null) {
        e.getControl().requestFocusInWindow();
      }
      return false;
    }

    if (showSummary) {
      try {
        // show a summary of what will be generated
        final int count = getStructure().getAbilityGenerationCount(createConfig());
        final String msg = String.format("%d spell %s will be created or updated.\nContinue?", count,
            (count == 1) ? "ability" : "abilities");
        int result = JOptionPane.showConfirmDialog(this, msg, "Confirm", JOptionPane.YES_NO_OPTION,
            JOptionPane.QUESTION_MESSAGE);
        return (result == JOptionPane.YES_OPTION);
      } catch (Exception e) {
        JOptionPane.showMessageDialog(this, e.getMessage(), "Error", JOptionPane.ERROR_MESSAGE);
        return false;
      }
    }
    return true;
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent e) {
    if (e.getSource() == bGenerate) {
      onGenerate();
    } else if (e.getSource() == bCancel) {
      onCancel();
    } else if (e.getSource() == bReset) {
      onReset();
    } else if (e.getSource() == bAdvanced) {
      onAdvanced();
    } else if (e.getSource() == miImport) {
      onImport();
    } else if (e.getSource() == miExport) {
      onExport();
    } else if (e.getSource() == cbDurationBlackList) {
      onDurationBlackListCheck(cbDuration.isSelected() && cbDurationBlackList.isSelected());
    } else if (e.getSource() == bDurationBlackList) {
      if (blackListDialog == null) {
        blackListDialog = new BlackListDialog(SwingUtilities.getWindowAncestor(this), getDurationBlackList(true));
      }
      blackListDialog.setVisible(true);
    } else if (e.getSource() instanceof JCheckBox) {
      onChecked((JCheckBox)e.getSource());
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  /** Imports configuration from a user-defined profile data file. */
  private void onImport() {
    final JFileChooser fc = new JFileChooser(Profile.getGameRoot().toFile());
    fc.setDialogTitle("Import profile");
    fc.setDialogType(JFileChooser.OPEN_DIALOG);
    fc.setFileSelectionMode(JFileChooser.FILES_ONLY);
    fc.setMultiSelectionEnabled(false);
    final FileNameExtensionFilter filter = new FileNameExtensionFilter("Spell Abilities Generator Files (*.sag)", "sag");
    fc.addChoosableFileFilter(filter);
    fc.setFileFilter(filter);
    if (fc.showOpenDialog(this) == JFileChooser.APPROVE_OPTION) {
      try {
        final File inFile = fc.getSelectedFile();
        final GeneratorConfig cfg = GeneratorConfig.load(inFile);
        applyConfig(cfg);
        JOptionPane.showMessageDialog(this, "Profile imported successfully:\n" + inFile.getPath(), "Information",
            JOptionPane.INFORMATION_MESSAGE);
      } catch (Exception e) {
        Logger.debug(e);
        final String msg;
        if (e.getMessage() != null && !e.getMessage().isEmpty()) {
          msg = e.getClass().getSimpleName() + " error: " + e.getMessage();
        } else {
          msg = "Unspecified " + e.getClass().getSimpleName() + " error";
        }
        JOptionPane.showMessageDialog(this, msg, "Error", JOptionPane.ERROR_MESSAGE);
      }
    }
  }

  /** Exports current state of input controls to a user-defined profile data file. */
  private void onExport() {
    final JFileChooser fc = new JFileChooser(Profile.getGameRoot().toFile());
    fc.setDialogTitle("Export profile");
    fc.setDialogType(JFileChooser.SAVE_DIALOG);
    fc.setFileSelectionMode(JFileChooser.FILES_ONLY);
    final FileNameExtensionFilter filter = new FileNameExtensionFilter("Spell Abilities Generator Files (*.sag)", "sag");
    fc.addChoosableFileFilter(filter);
    fc.setFileFilter(filter);
    if (fc.showSaveDialog(this) == JFileChooser.APPROVE_OPTION) {
      try {
        File outFile = fc.getSelectedFile();
        if (outFile.getName().indexOf('.') < 0) {
          // enforce file extension
          outFile = new File(outFile.getPath() + ".sag");
        }

        // checking for existing file
        if (outFile.exists()) {
          final String msg = "File already exists:\n" + outFile.getPath() + "\n\nOverwrite?";
          int result = JOptionPane.showConfirmDialog(this, msg, "Overwrite", JOptionPane.YES_NO_OPTION,
              JOptionPane.QUESTION_MESSAGE);
          if (result != JOptionPane.YES_OPTION) {
            return;
          }
        }

        final GeneratorConfig config = createConfig();
        config.save(outFile);
        JOptionPane.showMessageDialog(this, "Profile exported successfully:\n" + outFile.getPath(), "Information",
            JOptionPane.INFORMATION_MESSAGE);
      } catch (Exception e) {
        final String msg = (e.getMessage() != null && !e.getMessage().isEmpty()) ? e.getMessage()
            : "Unspecified " + e.getClass().getSimpleName() + " error";
        JOptionPane.showMessageDialog(this, msg, "Error", JOptionPane.ERROR_MESSAGE);
      }
    }
  }

  /** Closes the dialog and marks the operation as "accepted" if it passes config validation. */
  private void onGenerate() {
    if (validateInput(true)) {
      config = createConfig();
      getDialog().accept();
    }
  }

  /** Closes the dialog and marks the operation as "cancelled". */
  private void onCancel() {
    getDialog().cancel();
  }

  /** Resets all values to their defaults. */
  private void onReset() {
    reset();
    effectsListDialog = null;
  }

  /** Opens the "Advanced Configuration" dialog. */
  private void onAdvanced() {
    if (effectsListDialog == null) {
      effectsListDialog = new EffectsListDialog(getDialog());
    }
    effectsListDialog.setVisible(true);
  }

  /** Updates UI control states based on the specified black list checkbox selection state. */
  private void onDurationBlackListCheck(boolean selected) {
    bDurationBlackList.setEnabled(selected);
  }

  /** Handles checkbox state changes. */
  private void onChecked(JCheckBox cb) {
    if (cb == null) {
      return;
    }

    final Set<JComponent> set = linkedControls.get(cb);
    if (set == null) {
      return;
    }

    final boolean enabled = cb.isSelected();
    for (final JComponent c : set) {
      c.setEnabled(enabled);
    }

    if (cb == cbDuration) {
      onDurationBlackListCheck(cbDuration.isSelected() && cbDurationBlackList.isSelected());
    }
  }

  private void init() {
    // Pressing this button invokes prompt: This operation will create %d distinct ability structures. Continue?
    bGenerate = new JButton("Generate...");
    bGenerate.addActionListener(this);

    bCancel = new JButton("Cancel");
    bCancel.addActionListener(this);

    bReset = new JButton("Reset");
    bReset.setToolTipText("Reset all input fields, including advanced options, to their default values.");
    bReset.addActionListener(this);

    bAdvanced = new JButton("Advanced...");
    bAdvanced.addActionListener(this);

    miImport = new JMenuItem("Import...", Icons.ICON_OPEN_16.getIcon());
    miImport.addActionListener(this);
    miExport = new JMenuItem("Export...", Icons.ICON_SAVE_16.getIcon());
    miExport.addActionListener(this);
    bpmProfiles = new ButtonPopupMenu("Profile");
    bpmProfiles.addItem(miExport);
    bpmProfiles.addItem(miImport);
    bpmProfiles.setToolTipText("Import or export ability generation profiles.");
    bpmProfiles.setIcon(Icons.ICON_ARROW_UP_15.getIcon());
    bpmProfiles.setIconTextGap(8);

    final JLabel lLevelsPerAbil = new JLabel("Levels per ability:");
    tfLevelsPerAbil = new JFormattedTextField(NumberFormat.getIntegerInstance());
    tracker.register(InputControl.LEVELS_PER_ABIL, tfLevelsPerAbil);

    final JLabel lMaxLevels = new JLabel("Maximum level:");
    tfMaxLevels = new JFormattedTextField(NumberFormat.getIntegerInstance());
    tracker.register(InputControl.MAX_LEVELS, tfMaxLevels);

    final String baseValue = "Base value:";
    final String increment = "Increment:";
    final String perAbility = "per ability";
    final String incrementTooltip =
        "Use fractional numbers to fine-tune intervals. Results will be rounded down to the nearest integer.";

    cbRange = new JCheckBox("Range");
    cbRange.addActionListener(this);

    final JLabel lRangeBase = new JLabel(baseValue);
    registerControl(cbRange, lRangeBase);
    tfRangeBase = new JFormattedTextField(NumberFormat.getIntegerInstance());
    registerControl(cbRange, tfRangeBase);
    tracker.register(InputControl.RANGE_BASE, tfRangeBase);

    final JLabel lRangeInc = new JLabel(increment);
    lRangeInc.setToolTipText(incrementTooltip);
    registerControl(cbRange, lRangeInc);
    final JLabel lRangePerAbil = new JLabel(perAbility);
    registerControl(cbRange, lRangePerAbil);
    tfRangeInc = new JFormattedTextField(NumberFormat.getNumberInstance(Locale.ROOT));
    registerControl(cbRange, tfRangeInc);
    tracker.register(InputControl.RANGE_INC, tfRangeInc);

    cbCastingSpeed = new JCheckBox("Casting speed");
    cbCastingSpeed.addActionListener(this);

    final JLabel lCastingSpeedBase = new JLabel(baseValue);
    registerControl(cbCastingSpeed, lCastingSpeedBase);
    tfCastingSpeedBase = new JFormattedTextField(NumberFormat.getIntegerInstance());
    registerControl(cbCastingSpeed, tfCastingSpeedBase);
    tracker.register(InputControl.CASTING_SPEED_BASE, tfCastingSpeedBase);

    final JLabel lCastingSpeedInc = new JLabel(increment);
    lCastingSpeedInc.setToolTipText(incrementTooltip);
    registerControl(cbCastingSpeed, lCastingSpeedInc);
    final JLabel lCastingSpeedPerAbil = new JLabel(perAbility);
    registerControl(cbCastingSpeed, lCastingSpeedPerAbil);
    tfCastingSpeedInc = new JFormattedTextField(NumberFormat.getNumberInstance(Locale.ROOT));
    registerControl(cbCastingSpeed, tfCastingSpeedInc);
    tracker.register(InputControl.CASTING_SPEED_INC, tfCastingSpeedInc);

    cbDuration = new JCheckBox("Duration");
    cbDuration.addActionListener(this);

    final JLabel lDurationBase = new JLabel(baseValue);
    registerControl(cbDuration, lDurationBase);
    tfDurationBase = new JFormattedTextField(NumberFormat.getIntegerInstance());
    registerControl(cbDuration, tfDurationBase);
    tracker.register(InputControl.DURATION_BASE, tfDurationBase);

    final JLabel lDurationInc = new JLabel(increment);
    lDurationInc.setToolTipText(incrementTooltip);
    registerControl(cbDuration, lDurationInc);
    final JLabel lDurationPerAbil = new JLabel(perAbility);
    registerControl(cbDuration, lDurationPerAbil);
    tfDurationInc = new JFormattedTextField(NumberFormat.getNumberInstance(Locale.ROOT));
    registerControl(cbDuration, tfDurationInc);
    tracker.register(InputControl.DURATION_INC, tfDurationInc);

    final JLabel lDurationCutOff = new JLabel("Threshold:");
    lDurationCutOff.setToolTipText("Durations less than the entered number of seconds will be preserved.");
    registerControl(cbDuration, lDurationCutOff);
    tfDurationThreshold = new JFormattedTextField(NumberFormat.getNumberInstance(Locale.ROOT));
    registerControl(cbDuration, tfDurationThreshold);
    tracker.register(InputControl.DURATION_THRESHOLD, tfDurationThreshold);

    cbDurationBlackList = new JCheckBox("Use blacklist:", false);
    cbDurationBlackList.setToolTipText("Duration of listed effects will be preserved.");
    cbDurationBlackList.addActionListener(this);
    registerControl(cbDuration, cbDurationBlackList);

    bDurationBlackList = new JButton("Customize...");
    bDurationBlackList.setEnabled(false);
    bDurationBlackList.addActionListener(this);
    registerControl(cbDuration, bDurationBlackList);

    cbDiceCount = new JCheckBox("Dice count");
    cbDiceCount.addActionListener(this);

    final JLabel lDiceCountBase = new JLabel(baseValue);
    registerControl(cbDiceCount, lDiceCountBase);
    tfDiceCountBase = new JFormattedTextField(NumberFormat.getIntegerInstance());
    registerControl(cbDiceCount, tfDiceCountBase);
    tracker.register(InputControl.DICE_COUNT_BASE, tfDiceCountBase);

    final JLabel lDiceCountInc = new JLabel(increment);
    lDiceCountInc.setToolTipText(incrementTooltip);
    registerControl(cbDiceCount, lDiceCountInc);
    final JLabel lDiceCountPerAbil = new JLabel(perAbility);
    registerControl(cbDiceCount, lDiceCountPerAbil);
    tfDiceCountInc = new JFormattedTextField(NumberFormat.getNumberInstance(Locale.ROOT));
    registerControl(cbDiceCount, tfDiceCountInc);
    tracker.register(InputControl.DICE_COUNT_INC, tfDiceCountInc);

    cbDiceSize = new JCheckBox("Dice size");
    cbDiceSize.addActionListener(this);

    final JLabel lDiceSizeBase = new JLabel(baseValue);
    registerControl(cbDiceSize, lDiceSizeBase);
    tfDiceSizeBase = new JFormattedTextField(NumberFormat.getIntegerInstance());
    registerControl(cbDiceSize, tfDiceSizeBase);
    tracker.register(InputControl.DICE_SIZE_BASE, tfDiceSizeBase);

    final JLabel lDiceSizeInc = new JLabel(increment);
    lDiceSizeInc.setToolTipText(incrementTooltip);
    registerControl(cbDiceSize, lDiceSizeInc);
    final JLabel lDiceSizePerAbil = new JLabel(perAbility);
    registerControl(cbDiceSize, lDiceSizePerAbil);
    tfDiceSizeInc = new JFormattedTextField(NumberFormat.getNumberInstance(Locale.ROOT));
    registerControl(cbDiceSize, tfDiceSizeInc);
    tracker.register(InputControl.DICE_SIZE_INC, tfDiceSizeInc);

    cbSaveBonus = new JCheckBox("Save bonus");
    cbSaveBonus.addActionListener(this);

    final JLabel lSaveBonusBase = new JLabel(baseValue);
    registerControl(cbSaveBonus, lSaveBonusBase);
    tfSaveBonusBase = new JFormattedTextField(NumberFormat.getIntegerInstance());
    registerControl(cbSaveBonus, tfSaveBonusBase);
    tracker.register(InputControl.SAVE_BONUS_BASE, tfSaveBonusBase);

    final JLabel lSaveBonusInc = new JLabel(increment);
    lSaveBonusInc.setToolTipText(incrementTooltip);
    registerControl(cbSaveBonus, lSaveBonusInc);
    final JLabel lSaveBonusPerAbil = new JLabel(perAbility);
    registerControl(cbSaveBonus, lSaveBonusPerAbil);
    tfSaveBonusInc = new JFormattedTextField(NumberFormat.getNumberInstance(Locale.ROOT));
    registerControl(cbSaveBonus, tfSaveBonusInc);
    tracker.register(InputControl.SAVE_BONUS_INC, tfSaveBonusInc);

    // setting common text field behavior
    for (final InputControl key : tracker.getControls()) {
      final JFormattedTextField tf = tracker.get(key);
      if (tf != null) {
        tf.addMouseListener(tracker);
        tf.addKeyListener(tracker);
        tf.setText(Integer.toString(key.getDefaultValue()));
        try {
          tf.commitEdit();
        } catch (ParseException e) {
        }
      }
    }
    tracker.setInputDimension();

    cbSkipUnneeded = new JCheckBox("Skip abilities not meeting spell level requirements", true);

    final GridBagConstraints c = new GridBagConstraints();

    // configuration controls
    setLayout(new GridBagLayout());

    final JPanel mainPanel = new JPanel(new GridBagLayout());

    // new row: levels per ability; max. level
    int curRow = 0;
    int gapTop = 0;
    int gapGroup = 16;
    ViewerUtil.setGBC(c, 0, curRow, 2, 1, 1.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    mainPanel.add(lLevelsPerAbil, c);
    ViewerUtil.setGBC(c, 2, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfLevelsPerAbil, c);
    ViewerUtil.setGBC(c, 3, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lMaxLevels, c);
    ViewerUtil.setGBC(c, 4, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfMaxLevels, c);
    ViewerUtil.setGBC(c, 5, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(new JLabel(), c);

    // new row: range
    curRow++;
    gapTop = 24;
    ViewerUtil.setGBC(c, 0, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    mainPanel.add(cbRange, c);
    ViewerUtil.setGBC(c, 1, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lRangeBase, c);
    ViewerUtil.setGBC(c, 2, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfRangeBase, c);
    ViewerUtil.setGBC(c, 3, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lRangeInc, c);
    ViewerUtil.setGBC(c, 4, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfRangeInc, c);
    ViewerUtil.setGBC(c, 5, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(lRangePerAbil, c);

    // new row: casting speed
    curRow++;
    gapTop = 8;
    ViewerUtil.setGBC(c, 0, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    mainPanel.add(cbCastingSpeed, c);
    ViewerUtil.setGBC(c, 1, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lCastingSpeedBase, c);
    ViewerUtil.setGBC(c, 2, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfCastingSpeedBase, c);
    ViewerUtil.setGBC(c, 3, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lCastingSpeedInc, c);
    ViewerUtil.setGBC(c, 4, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfCastingSpeedInc, c);
    ViewerUtil.setGBC(c, 5, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(lCastingSpeedPerAbil, c);

    // new row: duration (base, increment)
    curRow++;
    gapTop = 8;
    ViewerUtil.setGBC(c, 0, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    mainPanel.add(cbDuration, c);
    ViewerUtil.setGBC(c, 1, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lDurationBase, c);
    ViewerUtil.setGBC(c, 2, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfDurationBase, c);
    ViewerUtil.setGBC(c, 3, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lDurationInc, c);
    ViewerUtil.setGBC(c, 4, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfDurationInc, c);
    ViewerUtil.setGBC(c, 5, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(lDurationPerAbil, c);

    // new row: duration (threshold, blacklist)
    curRow++;
    gapTop = 8;
    ViewerUtil.setGBC(c, 0, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    mainPanel.add(new JPanel(), c);
    ViewerUtil.setGBC(c, 1, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lDurationCutOff, c);
    ViewerUtil.setGBC(c, 2, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfDurationThreshold, c);
    ViewerUtil.setGBC(c, 3, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(cbDurationBlackList, c);
    ViewerUtil.setGBC(c, 4, curRow, 2, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(bDurationBlackList, c);

    // new row: dice count
    curRow++;
    gapTop = 8;
    ViewerUtil.setGBC(c, 0, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    mainPanel.add(cbDiceCount, c);
    ViewerUtil.setGBC(c, 1, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lDiceCountBase, c);
    ViewerUtil.setGBC(c, 2, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfDiceCountBase, c);
    ViewerUtil.setGBC(c, 3, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lDiceCountInc, c);
    ViewerUtil.setGBC(c, 4, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfDiceCountInc, c);
    ViewerUtil.setGBC(c, 5, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(lDiceCountPerAbil, c);

    // new row: dice size
    curRow++;
    gapTop = 8;
    ViewerUtil.setGBC(c, 0, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    mainPanel.add(cbDiceSize, c);
    ViewerUtil.setGBC(c, 1, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lDiceSizeBase, c);
    ViewerUtil.setGBC(c, 2, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfDiceSizeBase, c);
    ViewerUtil.setGBC(c, 3, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lDiceSizeInc, c);
    ViewerUtil.setGBC(c, 4, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfDiceSizeInc, c);
    ViewerUtil.setGBC(c, 5, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(lDiceSizePerAbil, c);

    // new row: save bonus
    curRow++;
    gapTop = 8;
    ViewerUtil.setGBC(c, 0, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    mainPanel.add(cbSaveBonus, c);
    ViewerUtil.setGBC(c, 1, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lSaveBonusBase, c);
    ViewerUtil.setGBC(c, 2, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfSaveBonusBase, c);
    ViewerUtil.setGBC(c, 3, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(gapTop, gapGroup, 0, 0), 0, 0);
    mainPanel.add(lSaveBonusInc, c);
    ViewerUtil.setGBC(c, 4, curRow, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(tfSaveBonusInc, c);
    ViewerUtil.setGBC(c, 5, curRow, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    mainPanel.add(lSaveBonusPerAbil, c);

    // new row: skip unneeded
    curRow++;
    gapTop = 16;
    ViewerUtil.setGBC(c, 0, curRow, 6, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    mainPanel.add(cbSkipUnneeded, c);

    // dialog buttons
    final JPanel buttonPanel = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    buttonPanel.add(bAdvanced, c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    buttonPanel.add(bReset, c);
    ViewerUtil.setGBC(c, 2, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    buttonPanel.add(bpmProfiles, c);
    ViewerUtil.setGBC(c, 3, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    buttonPanel.add(new JPanel(), c);
    ViewerUtil.setGBC(c, 4, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    buttonPanel.add(bGenerate, c);
    ViewerUtil.setGBC(c, 5, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    buttonPanel.add(bCancel, c);

    // put all together
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.CENTER, GridBagConstraints.BOTH,
        new Insets(0, 0, 0, 0), 0, 0);
    add(mainPanel, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 1.0, GridBagConstraints.CENTER, GridBagConstraints.HORIZONTAL,
        new Insets(12, 0, 0, 0), 0, 0);
    add(buttonPanel, c);

    for (final JCheckBox cb : linkedControls.keySet()) {
      onChecked(cb);
    }
  }

  /** Links the specified {@link JComponent} with a {@link JCheckBox}. */
  private void registerControl(JCheckBox checkBox, JComponent comp) {
    if (checkBox == null || comp == null) {
      return;
    }

    final Set<JComponent> set = linkedControls.computeIfAbsent(checkBox, cb -> new HashSet<>());
    if (!set.contains(comp)) {
      set.add(comp);
    }
  }

  // -------------------------- INNER CLASSES --------------------------

  /** Enum with keys that can be associated with any of the dialog input controls. */
  private static enum InputControl implements InputBounds {
    LEVELS_PER_ABIL(1, Byte.MAX_VALUE, 1),
    MAX_LEVELS(1, Byte.MAX_VALUE, 20),
    RANGE_BASE(0, Short.MAX_VALUE, 40),
    RANGE_INC(0, Short.MAX_VALUE, 0),
    CASTING_SPEED_BASE(0, Short.MAX_VALUE, 0),
    CASTING_SPEED_INC(Short.MIN_VALUE, Short.MAX_VALUE, 0),
    DURATION_BASE(0, Integer.MAX_VALUE, 0),
    DURATION_INC(0, Integer.MAX_VALUE, 6),
    DURATION_THRESHOLD(0, Integer.MAX_VALUE, 5),
    DICE_COUNT_BASE(0, Integer.MAX_VALUE, 0),
    DICE_COUNT_INC(0, Integer.MAX_VALUE, 0),
    DICE_SIZE_BASE(0, Integer.MAX_VALUE, 0),
    DICE_SIZE_INC(0, Integer.MAX_VALUE, 0),
    SAVE_BONUS_BASE(Integer.MIN_VALUE, Integer.MAX_VALUE, 0),
    SAVE_BONUS_INC(Integer.MIN_VALUE, Integer.MAX_VALUE, 0, 1),
    ;

    private final int min;
    private final int max;
    private final int defValue;
    private final int scale;

    InputControl(int min, int max, int defValue) {
      this(min, max, defValue, 0);
    }

    InputControl(int min, int max, int defValue, int scale) {
      this.min = min;
      this.max = max;
      this.defValue = defValue;
      this.scale = scale;
    }

    @Override
    public int getMinValue() {
      return min;
    }

    @Override
    public int getMaxValue() {
      return max;
    }

    @Override
    public int getDefaultValue() {
      return defValue;
    }

    @Override
    public int getScaleValue() {
      return scale;
    }
  }
}

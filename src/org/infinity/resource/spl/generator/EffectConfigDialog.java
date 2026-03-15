// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.spl.generator;

import java.awt.CardLayout;
import java.awt.Component;
import java.awt.Container;
import java.awt.Dialog;
import java.awt.Dimension;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.Window;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.ItemEvent;
import java.awt.event.ItemListener;
import java.awt.event.KeyEvent;
import java.text.NumberFormat;
import java.text.ParseException;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.Set;

import javax.swing.AbstractAction;
import javax.swing.DefaultComboBoxModel;
import javax.swing.JButton;
import javax.swing.JCheckBox;
import javax.swing.JComboBox;
import javax.swing.JComponent;
import javax.swing.JDialog;
import javax.swing.JFormattedTextField;
import javax.swing.JLabel;
import javax.swing.JOptionPane;
import javax.swing.JPanel;
import javax.swing.KeyStroke;
import javax.swing.UIManager;
import javax.swing.WindowConstants;
import javax.swing.border.Border;

import org.infinity.gui.ViewerUtil;
import org.infinity.resource.effects.BaseOpcode;
import org.infinity.resource.spl.generator.InputTracker.ValidationException;
import org.infinity.util.Misc;

/**
 *
 */
public class EffectConfigDialog extends JDialog implements ActionListener, ItemListener {
  /** Used internally to identify the respective parameter panels. */
  private enum Parameter {
    PARAMETER1, PARAMETER2, SPECIAL;

    /** Returns a unique identifier based on the {@link Parameter} and specified {@link EffectConfig.Mode} enum. */
    public String getIdentifier(EffectConfig.Mode mode) {
      return this.name() + '_' + Misc.orDefault(mode, EffectConfig.Mode.DWORD);
    }
  }

  /** Client property key for the panel comboboxes with an {@link Parameter} enum. */
  private static final String PROPERTY_PARAMETER = "Parameter";
  /** Client property key for the panel comboboxes with a reference to the associated parameter {@link JPanel}. */
  private static final String PROPERTY_PANEL = "Panel";

  private JComboBox<Opcode> cbOpcodes;
  private JComboBox<EffectConfig.Mode> cbParam1Modes;
  private JComboBox<EffectConfig.Mode> cbParam2Modes;
  private JComboBox<EffectConfig.Mode> cbSpecialModes;
  private JButton bAccept, bCancel;
  private EffectConfig config;

  public EffectConfigDialog(Window window, EffectConfig config) {
    super(window, "Effect Configuration", Dialog.ModalityType.DOCUMENT_MODAL);
    init();
    applyConfig(config);
  }

  /** Returns data as an {@link EffectConfig} object if accepted, {@code null} otherwise. */
  public EffectConfig getConfig() {
    return config;
  }

  /** Applies the specified configuration to the UI. Uses defaults if {@code null} is specified. */
  public void applyConfig(EffectConfig config) {
    if (config == null) {
      config = new EffectConfig();
    }

    // applying config to UI controls
    // opcode
    if (config.getOpcode() >= 0 && config.getOpcode() < cbOpcodes.getModel().getSize()) {
      cbOpcodes.setSelectedIndex(config.getOpcode());
    }

    // parameter 1
    EffectConfig.Mode mode = config.getParameter1Mode();
    cbParam1Modes.setSelectedIndex(mode.ordinal());
    ParameterPanel panel = getParameterPanel(cbParam1Modes, mode.ordinal());
    if (panel != null) {
      for (int i = 0, count = panel.getMode().getItemCount(); i < count; i++) {
        panel.setAttribute(i, config.getParameter1(i));
      }
    }

    // parameter 2
    mode = config.getParameter2Mode();
    cbParam2Modes.setSelectedIndex(mode.ordinal());
    panel = getParameterPanel(cbParam2Modes, mode.ordinal());
    if (panel != null) {
      for (int i = 0, count = panel.getMode().getItemCount(); i < count; i++) {
        panel.setAttribute(i, config.getParameter2(i));
      }
    }

    // special
    mode = config.getSpecialMode();
    cbSpecialModes.setSelectedIndex(mode.ordinal());
    panel = getParameterPanel(cbSpecialModes, mode.ordinal());
    if (panel != null) {
      for (int i = 0, count = panel.getMode().getItemCount(); i < count; i++) {
        panel.setAttribute(i, config.getSpecial(i));
      }
    }
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent e) {
    if (e.getSource() == bAccept) {
      onAccept();
    } else if (e.getSource() == bCancel) {
      onCancel();
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  // --------------------- Begin Interface ItemListener ---------------------

  @SuppressWarnings("unchecked")
  @Override
  public void itemStateChanged(ItemEvent e) {
    if (e.getSource() instanceof JComboBox<?> && e.getStateChange() == ItemEvent.SELECTED) {
      final JComboBox<EffectConfig.Mode> cb = (JComboBox<EffectConfig.Mode>)e.getSource();
      setParameterPanel(cb, cb.getSelectedIndex());
    }
  }

  // --------------------- End Interface ItemListener ---------------------

  /**
   * Helper method that returns the specified {@link ParameterPanel} that is associated with the specified combobox.
   *
   * @param comboBox  The {@link JComboBox} control.
   * @param itemIndex ComboBox item index to use.
   */
  private ParameterPanel getParameterPanel(JComboBox<EffectConfig.Mode> comboBox, int itemIndex) {
    if (comboBox == null) {
      return null;
    }

    final EffectConfig.Mode mode = comboBox.getItemAt(itemIndex);
    final Parameter attr = (Parameter)comboBox.getClientProperty(PROPERTY_PARAMETER);
    final JPanel cardPanel = (JPanel)comboBox.getClientProperty(PROPERTY_PANEL);
    if (mode != null && attr != null && cardPanel != null) {
      final String name = attr.getIdentifier(mode);
      for (final Component c : cardPanel.getComponents()) {
        if (Objects.equals(name, c.getName()) && c instanceof ParameterPanel) {
          return (ParameterPanel)c;
        }
      }
    }

    return null;
  }

  /**
   * Helper method that activates the {@link ParameterPanel} linked to the selected item of the specified combobox.
   *
   * @param comboBox  The {@link JComboBox} control.
   * @param itemIndex ComboBox item index to use.
   */
  private void setParameterPanel(JComboBox<EffectConfig.Mode> comboBox, int itemIndex) {
    if (comboBox == null) {
      return;
    }

    final EffectConfig.Mode mode = comboBox.getItemAt(itemIndex);
    final Parameter attr = (Parameter)comboBox.getClientProperty(PROPERTY_PARAMETER);
    final JPanel cardPanel = (JPanel)comboBox.getClientProperty(PROPERTY_PANEL);
    final CardLayout layout =
        (cardPanel != null && cardPanel.getLayout() instanceof CardLayout) ? (CardLayout)cardPanel.getLayout() : null;
    if (mode != null && attr != null && layout != null) {
      final String name = attr.getIdentifier(mode);
      layout.show(cardPanel, name);
    }
  }

  /** Accepts entered data and closes the dialog. */
  private void onAccept() {
    if (validateInput()) {
      config = createConfig();
      setVisible(false);
    }
  }

  /** Discards entered data and closes the dialog. */
  private void onCancel() {
    config = null;
    setVisible(false);
  }

  private boolean validateInput() {
    ParameterPanel panel = getParameterPanel(cbParam1Modes, cbParam1Modes.getSelectedIndex());
    if (panel != null) {
      if (!panel.validateInput()) {
        return false;
      }
    }

    panel = getParameterPanel(cbParam2Modes, cbParam2Modes.getSelectedIndex());
    if (panel != null) {
      if (!panel.validateInput()) {
        return false;
      }
    }

    panel = getParameterPanel(cbSpecialModes, cbSpecialModes.getSelectedIndex());
    if (panel != null) {
      return panel.validateInput();
    }

    return true;
  }

  private void init() {
    final DefaultComboBoxModel<Opcode> opcodesModel = new DefaultComboBoxModel<>();
    final int numOpcodes = BaseOpcode.getEffectNames().length;
    for (int i = 0; i < numOpcodes; i++) {
      opcodesModel.addElement(new Opcode(i));
    }
    cbOpcodes = new JComboBox<>(opcodesModel);
    cbOpcodes.setEditable(false);

    DefaultComboBoxModel<EffectConfig.Mode> modesModel = new DefaultComboBoxModel<>(EffectConfig.Mode.values());
    cbParam1Modes = new JComboBox<>(modesModel);
    cbParam1Modes.addItemListener(this);
    cbParam1Modes.setEditable(false);
    cbParam1Modes.putClientProperty(PROPERTY_PARAMETER, Parameter.PARAMETER1);

    modesModel = new DefaultComboBoxModel<>(EffectConfig.Mode.values());
    cbParam2Modes = new JComboBox<>(modesModel);
    cbParam2Modes.addItemListener(this);
    cbParam2Modes.setEditable(false);
    cbParam2Modes.putClientProperty(PROPERTY_PARAMETER, Parameter.PARAMETER2);

    modesModel = new DefaultComboBoxModel<>(EffectConfig.Mode.values());
    cbSpecialModes = new JComboBox<>(modesModel);
    cbSpecialModes.addItemListener(this);
    cbSpecialModes.setEditable(false);
    cbSpecialModes.putClientProperty(PROPERTY_PARAMETER, Parameter.SPECIAL);

    bAccept = new JButton("Accept");
    bAccept.addActionListener(this);

    bCancel = new JButton("Cancel");
    bCancel.addActionListener(this);

    final GridBagConstraints c = new GridBagConstraints();

    // selection panels
    final JPanel panelParam1Select = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    panelParam1Select.add(new JLabel("Parameter 1:"), c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelParam1Select.add(cbParam1Modes, c);

    final JPanel panelParam2Select = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    panelParam2Select.add(new JLabel("Parameter 2:"), c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelParam2Select.add(cbParam2Modes, c);

    final JPanel panelSpecialSelect = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    panelSpecialSelect.add(new JLabel("Special:"), c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelSpecialSelect.add(cbSpecialModes, c);

    // parameter panels
    final JPanel panelParam1Parameter = new JPanel(new CardLayout());
    for (final EffectConfig.Mode mode : EffectConfig.Mode.values()) {
      final ParameterPanel paramPanel = new ParameterPanel(mode);
      paramPanel.setName(Parameter.PARAMETER1.getIdentifier(mode));
      panelParam1Parameter.add(paramPanel, paramPanel.getName());
    }
    cbParam1Modes.putClientProperty(PROPERTY_PANEL, panelParam1Parameter);

    final JPanel panelParam2Parameter = new JPanel(new CardLayout());
    for (final EffectConfig.Mode mode : EffectConfig.Mode.values()) {
      final ParameterPanel paramPanel = new ParameterPanel(mode);
      paramPanel.setName(Parameter.PARAMETER2.getIdentifier(mode));
      panelParam2Parameter.add(paramPanel, paramPanel.getName());
    }
    cbParam2Modes.putClientProperty(PROPERTY_PANEL, panelParam2Parameter);

    final JPanel panelSpecialParameter = new JPanel(new CardLayout());
    for (final EffectConfig.Mode mode : EffectConfig.Mode.values()) {
      final ParameterPanel paramPanel = new ParameterPanel(mode);
      paramPanel.setName(Parameter.SPECIAL.getIdentifier(mode));
      panelSpecialParameter.add(paramPanel, paramPanel.getName());
    }
    cbSpecialModes.putClientProperty(PROPERTY_PANEL, panelSpecialParameter);

    // main configuration panel
    final JPanel panelMain = new JPanel(new GridBagLayout());

    // row 1: opcodes
    int row = 0;
    int gapTop = 0;
    ViewerUtil.setGBC(c, 0, row, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.NONE,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    panelMain.add(new JLabel("Opcode:"), c);
    ViewerUtil.setGBC(c, 1, row, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    panelMain.add(cbOpcodes, c);

    // row 2: parameter 1
    row++;
    gapTop = 16;
    ViewerUtil.setGBC(c, 0, row, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    panelMain.add(panelParam1Select, c);
    ViewerUtil.setGBC(c, 1, row, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    panelMain.add(panelParam1Parameter, c);

    // row 3: parameter 2
    row++;
    gapTop = 16;
    ViewerUtil.setGBC(c, 0, row, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    panelMain.add(panelParam2Select, c);
    ViewerUtil.setGBC(c, 1, row, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    panelMain.add(panelParam2Parameter, c);

    // row 4: special
    row++;
    gapTop = 16;
    ViewerUtil.setGBC(c, 0, row, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 0, 0, 0), 0, 0);
    panelMain.add(panelSpecialSelect, c);
    ViewerUtil.setGBC(c, 1, row, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(gapTop, 8, 0, 0), 0, 0);
    panelMain.add(panelSpecialParameter, c);

    // button panel
    final JPanel panelButtons = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    panelButtons.add(new JPanel(), c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 8), 0, 0);
    panelButtons.add(bAccept, c);
    ViewerUtil.setGBC(c, 2, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    panelButtons.add(bCancel, c);
    ViewerUtil.setGBC(c, 3, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    panelButtons.add(new JPanel(), c);

    // putting it all together
    final Container contentPane = getContentPane();
    contentPane.setLayout(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.BOTH,
        new Insets(8, 8, 0, 8), 0, 0);
    contentPane.add(panelMain, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(16, 8, 8, 8), 0, 0);
    contentPane.add(panelButtons, c);

    setParameterPanel(cbParam1Modes, cbParam1Modes.getSelectedIndex());
    setParameterPanel(cbParam2Modes, cbParam2Modes.getSelectedIndex());
    setParameterPanel(cbSpecialModes, cbSpecialModes.getSelectedIndex());

    pack();
    setResizable(false);
    setLocationRelativeTo(getParent());

    // "Closing" the dialog only makes it invisible
    setDefaultCloseOperation(WindowConstants.HIDE_ON_CLOSE);

    getRootPane().setDefaultButton(bAccept);

    getRootPane().getInputMap(JComponent.WHEN_IN_FOCUSED_WINDOW).put(KeyStroke.getKeyStroke(KeyEvent.VK_ESCAPE, 0),
        getRootPane());
    getRootPane().getActionMap().put(getRootPane(), new AbstractAction() {
      @Override
      public void actionPerformed(ActionEvent event) {
        onCancel();
      }
    });
  }

  /** Initializes a new {@link EffectConfig} instance with the current dialog data and returns it. */
  private EffectConfig createConfig() {
    final EffectConfig config = new EffectConfig();

    // opcode
    final Opcode opcode = (Opcode)cbOpcodes.getSelectedItem();
    if (opcode != null) {
      config.setOpcode(opcode.intValue());
    }

    // parameter 1
    ParameterPanel panel = getParameterPanel(cbParam1Modes, cbParam1Modes.getSelectedIndex());
    if (panel != null) {
      config.setParameter1Mode(panel.getMode());
      for (int i = 0, count = panel.getMode().getItemCount(); i < count; i++) {
        config.getParameter1(i).setAttribute(panel.getAttribute(i));
      }
    }

    // parameter 2
    panel = getParameterPanel(cbParam2Modes, cbParam2Modes.getSelectedIndex());
    if (panel != null) {
      config.setParameter2Mode(panel.getMode());
      for (int i = 0, count = panel.getMode().getItemCount(); i < count; i++) {
        config.getParameter2(i).setAttribute(panel.getAttribute(i));
      }
    }

    // special
    panel = getParameterPanel(cbSpecialModes, cbSpecialModes.getSelectedIndex());
    if (panel != null) {
      config.setSpecialMode(panel.getMode());
      for (int i = 0, count = panel.getMode().getItemCount(); i < count; i++) {
        config.getSpecial(i).setAttribute(panel.getAttribute(i));
      }
    }

    return config;
  }

  // -------------------------- INNER CLASSES --------------------------

  /** Creates and manages a single parameter panel of any given {@link EffectConfig.Mode}. */
  private static class ParameterPanel extends JPanel implements ActionListener {
    private final HashMap<JCheckBox, Set<JComponent>> linkedControls = new HashMap<>();
    private final InputTracker<InputControl> tracker = new InputTracker<>(InputControl.class);

    private final List<JCheckBox> selections = new ArrayList<>();
    private final List<JFormattedTextField> baseInputFields = new ArrayList<>();
    private final List<JFormattedTextField> incrementInputFields = new ArrayList<>();

    private final EffectConfig.Mode mode;

    public ParameterPanel(EffectConfig.Mode mode) {
      super(new GridBagLayout());
      this.mode = Objects.requireNonNull(mode);
      init();
    }

    /** Returns the input {@link EffectConfig.Mode} associated with this panel. */
    public EffectConfig.Mode getMode() {
      return mode;
    }

    /**
     * Returns attribute values of the specified parameter item as a {@link GeneratorAttribute} object.
     *
     * @param index Parameter item index.
     * @return The initialized {@link GeneratorAttribute} object.
     * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid parameter items can be
     *                                     determined by {@link EffectConfig.Mode#getItemCount()}.
     * @see #getMode()
     */
    public GeneratorAttribute getAttribute(int index) {
      GeneratorAttribute retVal = null;
      if (index >= 0 && index < mode.getItemCount()) {
        retVal = new GeneratorAttribute(getBaseValue(index), getIncrementValue(index), isItemEnabled(index));
      }
      return retVal;
    }

    /**
     * Applies attribute values of the {@link GeneratorAttribute} object to the panel controls.
     *
     * @param index Parameter item index.
     * @param attribute {@link GeneratorAttribute} object with parameter values to apply.
     * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid parameter items can be
     *                                     determined by {@link EffectConfig.Mode#getItemCount()}.
     * @see #getMode()
     */
    public void setAttribute(int index, GeneratorAttribute attribute) {
      if (attribute != null) {
        setItemEnabled(index, attribute.isEnabled());
        setBaseValue(index, attribute.getBase());
        setIncrementValue(index, attribute.getIncrement());
        onChecked(selections.get(index));
      }
    }

    /**
     * Returns whether the specified parameter item should be considered by the ability generation.
     *
     * @param index Parameter item index.
     * @return {@code true} if the parameter item is enabled
     * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid parameter items can be
     *                                     determined by {@link EffectConfig.Mode#getItemCount()}.
     * @see #getMode()
     */
    public boolean isItemEnabled(int index) {
      return selections.get(index).isSelected();
    }

    /**
     * Specifies whether the specified parameter item should be considered by the ability generation.
     *
     * @param index  Parameter item index.
     * @param enable The enabled state to set.
     * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid parameter items can be
     *                                     determined by {@link EffectConfig.Mode#getItemCount()}.
     * @see #getMode()
     */
    public void setItemEnabled(int index, boolean enable) {
      selections.get(index).setSelected(enable);
    }

    /**
     * Returns the base value of the given parameter item.
     *
     * @param index Parameter item index.
     * @return Base value defined by the specified parameter item. Returns {@code null} if no valid number is defined.
     * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid parameter items can be
     *                                     determined by {@link EffectConfig.Mode#getItemCount()}.
     * @see #getMode()
     */
    public Integer getBaseValue(int index) {
      final InputControl key = tracker.getKey(baseInputFields.get(index));
      if (key != null) {
        final Number value = tracker.getValue(key);
        if (value != null) {
          return value.intValue();
        }
      }
      return null;
    }

    /**
     * Sets the base value of the given parameter item.
     *
     * @param index    Parameter item index.
     * @param newValue New parameter value.
     * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid parameter items can be
     *                                     determined by {@link EffectConfig.Mode#getItemCount()}.
     * @see #getMode()
     */
    public void setBaseValue(int index, int newValue) {
      baseInputFields.get(index).setText(Integer.toString(newValue));
    }

    /**
     * Returns the increment per ability value of the given parameter item.
     *
     * @param index Parameter item index.
     * @return Increment per ability value defined by the specified parameter item. Returns {@code null} if no valid
     *         number is defined.
     * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid parameter items can be
     *                                     determined by {@link EffectConfig.Mode#getItemCount()}.
     * @see #getMode()
     */
    public Double getIncrementValue(int index) {
      final InputControl key = tracker.getKey(incrementInputFields.get(index));
      if (key != null) {
        final Number value = tracker.getValue(key);
        if (value != null) {
          return value.doubleValue();
        }
      }
      return null;
    }

    /**
     * Sets the increment per ability value of the given parameter item.
     *
     * @param index    Parameter item index.
     * @param newValue New parameter value.
     * @throws IndexOutOfBoundsException if specified index is out of bounds. Number of valid parameter items can be
     *                                     determined by {@link EffectConfig.Mode#getItemCount()}.
     * @see #getMode()
     */
    public void setIncrementValue(int index, double newValue) {
      incrementInputFields.get(index).setText(Double.toString(newValue));
    }

    /** Checks whether current configuration of the panel is valid in the current context. */
    public boolean validateInput() {
      try {
        tracker.validateInput(true);
      } catch (ValidationException e) {
        JOptionPane.showMessageDialog(this, e.getMessage(), "Error", JOptionPane.ERROR_MESSAGE);
        if (e.getControl() != null) {
          e.getControl().requestFocusInWindow();
        }
        return false;
      }
      return true;
    }

    // --------------------- Begin Interface ActionListener ---------------------

    @Override
    public void actionPerformed(ActionEvent e) {
      if (e.getSource() instanceof JCheckBox) {
        onChecked((JCheckBox)e.getSource());
      }
    }

    // --------------------- End Interface ActionListener ---------------------

    /** Handles checkbox state changes. */
    private void onChecked(JCheckBox cb) {
      if (cb == null) {
        return;
      }

      final Set<JComponent> set = linkedControls.get(cb);
      if (set == null) {
        return;
      }

      final boolean selected = cb.isSelected();
      for (final JComponent c : set) {
        c.setEnabled(selected);
      }
    }

    private void init() {
      // initializing controls
      final JLabel lBase = new JLabel("Base value:");
      final JLabel lIncrement = new JLabel("Increment:");
      final JLabel lAbility = new JLabel("per ability");

      // preparing prototype string for input field width calculation
      // approxiated max. number length (with thousands separators)
      int protoLen = (mode.getItemSize() * 8) / 3;
      protoLen += (protoLen / 3);
      final char[] charArray = new char[protoLen];
      Arrays.fill(charArray, 'X');
      final String prototype = new String(charArray);

      for (int i = 0; i < mode.getItemCount(); i++) {
        final int index = i + 1;
        final JCheckBox cb = new JCheckBox(mode.getLabel() + " " + index, false);
        cb.addActionListener(this);
        selections.add(cb);
        final Dimension dimCheck = cb.getPreferredSize();

        final JFormattedTextField tfBase = new JFormattedTextField(NumberFormat.getIntegerInstance(Locale.ROOT));
        tracker.register(InputControl.getBaseInputControl(mode, i), tfBase);
        registerControl(cb, tfBase);
        baseInputFields.add(tfBase);
        final Dimension dimBase = Misc.getPrototypeSize(tfBase, prototype);

        final JFormattedTextField tfInc = new JFormattedTextField(NumberFormat.getNumberInstance(Locale.ROOT));
        tracker.register(InputControl.getIncrementInputControl(mode, i), tfInc);
        registerControl(cb, tfInc);
        incrementInputFields.add(tfInc);
        final Dimension dimInc = Misc.getPrototypeSize(tfInc, prototype);

        // adjusting preferred width to a common value
        int maxWidth = Math.max(dimCheck.width, Math.max(dimBase.width, dimInc.width));
        dimCheck.width = maxWidth;
        dimBase.width = maxWidth;
        dimInc.width = maxWidth;
        cb.setPreferredSize(dimCheck);
        tfBase.setPreferredSize(dimBase);
        tfInc.setPreferredSize(dimInc);
      }

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

      // building table layout
      final GridBagConstraints c = new GridBagConstraints();
      final JPanel mainPanel = new JPanel(new GridBagLayout());

      // row 1: checkboxes (x, checkboxes..., x
      int row = 0;
      int col = 0;
      int gapTop = 0;
      ViewerUtil.setGBC(c, col++, row, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(gapTop, 0, 0, 0), 0, 0);
      mainPanel.add(new JLabel(), c);
      for (int i = 0; i < mode.getItemCount(); i++) {
        ViewerUtil.setGBC(c, col++, row, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
            new Insets(gapTop, 8, 0, 0), 0, 0);
        mainPanel.add(selections.get(i), c);
      }
      ViewerUtil.setGBC(c, col++, row, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(gapTop, 8, 0, 0), 0, 0);
      mainPanel.add(new JLabel(), c);

      // row 2: base value (label, inputfields..., x)
      row++;
      col = 0;
      gapTop = 8;
      ViewerUtil.setGBC(c, col++, row, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(gapTop, 0, 0, 0), 0, 0);
      mainPanel.add(lBase, c);
      for (int i = 0; i < mode.getItemCount(); i++) {
        ViewerUtil.setGBC(c, col++, row, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
            new Insets(gapTop, 8, 0, 0), 0, 0);
        mainPanel.add(baseInputFields.get(i), c);
      }
      ViewerUtil.setGBC(c, col++, row, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(gapTop, 8, 0, 0), 0, 0);
      mainPanel.add(new JLabel(), c);

      // row 3: increment  (label, inputfields..., label)
      row++;
      col = 0;
      gapTop = 8;
      ViewerUtil.setGBC(c, col++, row, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(gapTop, 0, 0, 0), 0, 0);
      mainPanel.add(lIncrement, c);
      for (int i = 0; i < mode.getItemCount(); i++) {
        ViewerUtil.setGBC(c, col++, row, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
            new Insets(gapTop, 8, 0, 0), 0, 0);
        mainPanel.add(incrementInputFields.get(i), c);
      }
      ViewerUtil.setGBC(c, col++, row, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(gapTop, 8, 0, 0), 0, 0);
      mainPanel.add(lAbility, c);

      // adding to root panel
      ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.LINE_START, GridBagConstraints.BOTH,
          new Insets(8, 8, 8, 8), 0, 0);
      add(mainPanel, c);

      // using default TitledBorder style
      final Border border = UIManager.getBorder("TitledBorder.border");
      if (border != null) {
        setBorder(border);
      }

      for (final JCheckBox cb : selections) {
        onChecked(cb);
      }
    }

    /** Links the specified {@link JComponent} with a {@link JCheckBox}. */
    private void registerControl(JCheckBox checkBox, JComponent comp) {
      if (checkBox == null || comp == null) {
        return;
      }

      final Set<JComponent> set = linkedControls.computeIfAbsent(checkBox, cb -> new HashSet<>());
      set.add(comp);
    }
  }

  /** Enum with keys that can be associated with any of the dialog input controls in a {@link ParameterPanel}. */
  private enum InputControl implements InputBounds {
    DWORD_BASE(Integer.MIN_VALUE, Integer.MAX_VALUE, 0),
    DWORD_INC(Integer.MIN_VALUE, Integer.MAX_VALUE, 0),

    WORD_BASE_1(Short.MIN_VALUE, 0xffff, 0),
    WORD_INC_1(Short.MIN_VALUE, 0xffff, 0),
    WORD_BASE_2(Short.MIN_VALUE, 0xffff, 0),
    WORD_INC_2(Short.MIN_VALUE, 0xffff, 0),

    BYTE_BASE_1(Byte.MIN_VALUE, 0xff, 0),
    BYTE_INC_1(Byte.MIN_VALUE, 0xff, 0),
    BYTE_BASE_2(Byte.MIN_VALUE, 0xff, 0),
    BYTE_INC_2(Byte.MIN_VALUE, 0xff, 0),
    BYTE_BASE_3(Byte.MIN_VALUE, 0xff, 0),
    BYTE_INC_3(Byte.MIN_VALUE, 0xff, 0),
    BYTE_BASE_4(Byte.MIN_VALUE, 0xff, 0),
    BYTE_INC_4(Byte.MIN_VALUE, 0xff, 0),
    ;

    /** Helper method that returns the base {@link InputControl} based on the specified parameters. */
    public static InputControl getBaseInputControl(EffectConfig.Mode mode, int index) {
      switch (mode) {
        case BYTE:
          switch (index) {
            case 1:
              return BYTE_BASE_2;
            case 2:
              return BYTE_BASE_3;
            case 3:
              return BYTE_BASE_4;
            default:
              return BYTE_BASE_1;
          }
        case WORD:
          return (index == 1) ? WORD_BASE_2 : WORD_BASE_1;
        default:
          return DWORD_BASE;
      }
    }

    /** Helper method that returns the increment {@link InputControl} based on the specified parameters. */
    public static InputControl getIncrementInputControl(EffectConfig.Mode mode, int index) {
      switch (mode) {
        case BYTE:
          switch (index) {
            case 1:
              return BYTE_INC_2;
            case 2:
              return BYTE_INC_3;
            case 3:
              return BYTE_INC_4;
            default:
              return BYTE_INC_1;
          }
        case WORD:
          return (index == 1) ? WORD_INC_2 : WORD_INC_1;
        default:
          return DWORD_INC;
      }
    }

    private final int min;
    private final int max;
    private final int defValue;

    InputControl(int min, int max, int defValue) {
      this.min = min;
      this.max = max;
      this.defValue = defValue;
    }

    /** Returns the lower bound for this control. */
    @Override
    public int getMinValue() {
      return min;
    }

    /** Returns the upper bound for this control. */
    @Override
    public int getMaxValue() {
      return max;
    }

    /** Returns the default value for this control. */
    @Override
    public int getDefaultValue() {
      return defValue;
    }
  }
}

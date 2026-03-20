// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter;

import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;

import javax.swing.JButton;
import javax.swing.JCheckBox;
import javax.swing.JPanel;

import org.infinity.gui.ViewerUtil;

/**
 * Button panel for the converter dialog buttons.
 */
public class ConvertButtonsPanel extends AbstractConvertPanel implements ActionListener {
  /** "Start Conversion" button was clicked. */
  public static final UpdateReason REASON_CONVERT = new UpdateReason(1);
  /** "Cancel" button was clicked. */
  public static final UpdateReason REASON_CANCEL = new UpdateReason(2);

  private JCheckBox cbCloseOnExit;
  private JButton bConvert, bCancel;

  public ConvertButtonsPanel() {
    super(new GridBagLayout());
    init();
  }

  /** Returns whether the "Close dialog after conversion" checkbox is selected. */
  public boolean isCloseOnExit() {
    return cbCloseOnExit.isSelected();
  }

  /** Sets whether the "Close dialog after conversion" checkbox is selected. */
  public void setCloseOnExit(boolean set) {
    cbCloseOnExit.setSelected(set);
  }

  /** Returns whether the "Close dialog after conversion" checkbox is enabled. */
  public boolean isCloseOnExitEnabled() {
    return cbCloseOnExit.isEnabled();
  }

  /** Sets whether the "Close dialog after conversion" checkbox is enabled. */
  public void setCloseOnExitEnabled(boolean enable) {
    cbCloseOnExit.setEnabled(enable);
  }

  /** Returns whether the "Start Conversion" button is enabled. */
  public boolean isConvertButtonEnabled() {
    return bConvert.isEnabled();
  }

  /** Sets whether the "Start Conversion" button is enabled. */
  public void setConvertButtonEnabled(boolean enable) {
    bConvert.setEnabled(enable);
  }

  /** Returns whether the "Cancel" button is enabled. */
  public boolean isCancelButtonEnabled() {
    return bCancel.isEnabled();
  }

  /** Sets whether the "Cancel" button is enabled. */
  public void setCancelButtonEnabled(boolean enable) {
    bCancel.setEnabled(enable);
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent e) {
    if (e.getSource() == bConvert) {
      onConvert();
    } else if (e.getSource() == bCancel) {
      onCancel();
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  private void onConvert() {
    firePanelUpdate(REASON_CONVERT);
  }

  private void onCancel() {
    firePanelUpdate(REASON_CANCEL);
  }

  private void init() {
    cbCloseOnExit = new JCheckBox("Close dialog after conversion", true);

    bConvert = new JButton("Start Conversion");
    bConvert.addActionListener(this);
    Insets insets = bConvert.getInsets();
    insets.set(insets.top + 2, insets.left, insets.bottom + 2, insets.right);
    bConvert.setMargin(insets);

    bCancel = new JButton("Cancel");
    bCancel.addActionListener(this);
    insets = bCancel.getInsets();
    insets.set(insets.top + 2, insets.left, insets.bottom + 2, insets.right);
    bCancel.setMargin(insets);

    final GridBagConstraints c = new GridBagConstraints();

    final JPanel panelButtons = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    panelButtons.add(cbCloseOnExit, c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    panelButtons.add(bConvert, c);
    ViewerUtil.setGBC(c, 2, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_END, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    panelButtons.add(bCancel, c);

    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 4, 4, 4), 0, 0);
    add(panelButtons, c);
  }
}

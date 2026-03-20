// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.mos;

import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;

import javax.swing.JButton;
import javax.swing.JComboBox;
import javax.swing.JLabel;
import javax.swing.JOptionPane;
import javax.swing.JSpinner;
import javax.swing.SpinnerNumberModel;
import javax.swing.event.ChangeEvent;
import javax.swing.event.ChangeListener;

import org.infinity.gui.ViewerUtil;
import org.infinity.gui.converter.AbstractConvertPanel;
import org.infinity.gui.converter.pvrz.CompressionType;
import org.infinity.util.Misc;

/** Subpanel with MOS V2-specific options. */
class MosV2OptionsPanel extends AbstractConvertPanel implements ActionListener, ChangeListener {
  private JSpinner spPvrzStartIndex;
  private JComboBox<CompressionType> cbCompression;
  private JButton bCompressionHelp;
  private JLabel lPvrzInfo;

  public MosV2OptionsPanel() {
    super(new GridBagLayout());
    init();
  }

  /** Returns the currently specified PVRZ start index. */
  public int getPvrzIndex() {
    final SpinnerNumberModel model = (SpinnerNumberModel)spPvrzStartIndex.getModel();
    return model.getNumber().intValue();
  }

  /** Sets a new PVRZ start index. */
  public void setPvrzIndex(int newValue) {
    final SpinnerNumberModel model = (SpinnerNumberModel)spPvrzStartIndex.getModel();
    final int min = ((Number)model.getMinimum()).intValue();
    final int max = ((Number)model.getMaximum()).intValue();
    newValue = Misc.clamp(newValue, min, max);
    model.setValue(newValue);
  }

  /** Returns the currently selected compression type. */
  public CompressionType getCompressionType() {
    final int index = cbCompression.getSelectedIndex();
    if (index >= 0) {
      return cbCompression.getModel().getElementAt(index);
    } else {
      return CompressionType.AUTO;
    }
  }

  /** Sets the currently selected compression type. */
  public void setCompressionType(CompressionType type) {
    if (type == null) {
      type = CompressionType.AUTO;
    }
    cbCompression.setSelectedItem(type);
  }

  /** Resets the panel to its initial state. */
  public void reset() {
    setPvrzIndex(0);
    setCompressionType(null);
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent e) {
    if (e.getSource() == bCompressionHelp) {
      JOptionPane.showMessageDialog(this, CompressionType.getHelp(), "About Compression Types",
          JOptionPane.INFORMATION_MESSAGE);
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  // --------------------- Begin Interface ChangeListener ---------------------

  @Override
  public void stateChanged(ChangeEvent e) {
    if (e.getSource() == spPvrzStartIndex) {
      lPvrzInfo.setText(getPvrzInfoString(getPvrzIndex()));
    }
  }

  // --------------------- End Interface ChangeListener ---------------------

  private void init() {
    final JLabel lPvrzStartIndex = new JLabel("PVRZ index starts at:");
    spPvrzStartIndex = new JSpinner(new SpinnerNumberModel(0, 0, 99999, 1));
    spPvrzStartIndex.setToolTipText("Enter a number between 0 and 99999");
    spPvrzStartIndex.addChangeListener(this);

    final JLabel lCompression = new JLabel("Compression type:");
    cbCompression = new JComboBox<>(CompressionType.values());
    cbCompression.setSelectedIndex(0);
    bCompressionHelp = new JButton("?");
    bCompressionHelp.setToolTipText("About compression types");
    bCompressionHelp.addActionListener(this);
    final Insets insets = bCompressionHelp.getMargin();
    insets.left = insets.right = 4;
    bCompressionHelp.setMargin(insets);

    lPvrzInfo = new JLabel(getPvrzInfoString(getPvrzIndex()));

    final GridBagConstraints c = new GridBagConstraints();
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    add(lPvrzStartIndex, c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    add(spPvrzStartIndex, c);
    ViewerUtil.setGBC(c, 2, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 16, 0, 0), 0, 0);
    add(lCompression, c);
    ViewerUtil.setGBC(c, 3, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    add(cbCompression, c);
    ViewerUtil.setGBC(c, 4, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    add(bCompressionHelp, c);

    ViewerUtil.setGBC(c, 0, 1, 5, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(4, 0, 0, 0), 0, 0);
    add(lPvrzInfo, c);

    reset();
  }

  /** Returns a string that lists potential generated PVRZ files generated, based on the given index. */
  private static String getPvrzInfoString(int index) {
    return String.format("Resulting in MOS%04d.PVRZ, MOS%04d.PVRZ, ...", index, index + 1);
  }
}
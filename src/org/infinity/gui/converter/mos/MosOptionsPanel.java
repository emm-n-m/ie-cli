// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.mos;

import java.awt.CardLayout;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.ItemEvent;
import java.awt.event.ItemListener;

import javax.swing.JButton;
import javax.swing.JComboBox;
import javax.swing.JLabel;
import javax.swing.JOptionPane;
import javax.swing.JPanel;

import org.infinity.gui.ViewerUtil;
import org.infinity.gui.converter.AbstractConvertPanel;

/** Subpanel that provides access to specific MOS V1 and MOS V2 subpanels. */
class MosOptionsPanel extends AbstractConvertPanel implements ActionListener, ItemListener {
  /** Used internally to identify the selected MOS version. */
  private enum MosVersion {
    V1("Legacy (V1)"),
    V2("PVRZ-based (V2)"),
    ;

    private final String label;
    MosVersion(String label) {
      this.label = label;
    }

    @SuppressWarnings("unused")
    public String getLabel() {
      return label;
    }

    @Override
    public String toString() {
      return label;
    }
  }

  private static final String MOS_VERSION_HELP =
      '"' + MosVersion.V1.toString() + "\" is the original MOS format supported by all available\n"
    + "Infinity Engine games. Graphics data is stored in the MOS file directly.\n"
    + "Each (64x64 pixel) block is limited to a 256 color table.\n\n"
    + '"' + MosVersion.V2.toString() + "\" uses a new format that is only compatible with the\n"
    + "Enhanced Editions. Graphics data is stored separately in PVRZ files,\n"
    + "is not limited to 256 colors, and supports alpha-blending.";

  private JComboBox<MosVersion> cbVersion;
  private JButton bVersionHelp;
  private JPanel optionsPanel;
  private MosV1OptionsPanel mosV1Panel;
  private MosV2OptionsPanel mosV2Panel;

  public MosOptionsPanel() {
    super(new GridBagLayout(), "Options");
    init();
  }

  /** Returns whether the legacy (palette-based) MOS version is selected. */
  public boolean isLegacyVersionSelected() {
    return cbVersion.getSelectedIndex() == 0;
  }

  /**
   * Specify {@code true} to select legacy (palette-based) MOS version or {@code false} to select PVRZ-based MOS
   * version.
   */
  public void setLegacyVersionSelected(boolean set) {
    cbVersion.setSelectedIndex(set ? 0 : 1);
  }

  /** Returns the {@link MosV1OptionsPanel} instance. */
  public MosV1OptionsPanel getMosV1Options() {
    return mosV1Panel;
  }

  /** Returns the {@link MosV2OptionsPanel} instance. */
  public MosV2OptionsPanel getMosV2Options() {
    return mosV2Panel;
  }

  /** Resets the panel to its initial state. */
  public void reset() {
    setLegacyVersionSelected(true);
    mosV1Panel.reset();
    mosV2Panel.reset();
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent e) {
    if (e.getSource() == bVersionHelp) {
      JOptionPane.showMessageDialog(ViewerUtil.getWindowAncestor(this), MOS_VERSION_HELP, "About MOS versions",
          JOptionPane.INFORMATION_MESSAGE);
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  // --------------------- Begin Interface ItemListener ---------------------

  @Override
  public void itemStateChanged(ItemEvent e) {
    if (e.getSource() == cbVersion) {
      onMosVersionChanged();
    }
  }

  // --------------------- End Interface ItemListener ---------------------

  /** Enables the panel associated with the currently selected combobox item. */
  private void onMosVersionChanged() {
    final int index = cbVersion.getSelectedIndex();
    if (index < 0) {
      return;
    }

    final MosVersion version = cbVersion.getModel().getElementAt(index);
    final CardLayout layout = (CardLayout)optionsPanel.getLayout();
    if (version != null && layout != null) {
      layout.show(optionsPanel, version.name());
    }
  }

  private void init() {
    final JLabel lVersion = new JLabel("Select MOS version:");

    cbVersion = new JComboBox<>(MosVersion.values());
    cbVersion.setSelectedIndex(0);
    cbVersion.addItemListener(this);

    bVersionHelp = new JButton("?");
    bVersionHelp.setToolTipText("About MOS versions...");
    bVersionHelp.addActionListener(this);
    final Insets insets = bVersionHelp.getMargin();
    insets.left = insets.right = 4;
    bVersionHelp.setMargin(insets);

    mosV1Panel = new MosV1OptionsPanel();
    mosV2Panel = new MosV2OptionsPanel();
    optionsPanel = new JPanel(new CardLayout());

    final GridBagConstraints c = new GridBagConstraints();

    // setting up card panels
    optionsPanel.add(mosV1Panel, MosVersion.V1.name());
    optionsPanel.add(mosV2Panel, MosVersion.V2.name());

    final JPanel panelMain = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    panelMain.add(lVersion, c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    panelMain.add(cbVersion, c);
    ViewerUtil.setGBC(c, 2, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 8, 0, 0), 0, 0);
    panelMain.add(bVersionHelp, c);

    ViewerUtil.setGBC(c, 0, 1, 3, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelMain.add(optionsPanel, c);

    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 4, 4, 4), 0, 0);
    add(panelMain, c);
  }
}
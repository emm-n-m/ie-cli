// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.spl.generator;

import java.awt.Container;
import java.awt.Dialog;
import java.awt.Dimension;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.Window;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.KeyEvent;
import java.awt.event.MouseEvent;
import java.awt.event.MouseListener;
import java.util.Collection;
import java.util.Set;
import java.util.TreeSet;

import javax.swing.AbstractAction;
import javax.swing.DefaultComboBoxModel;
import javax.swing.DefaultListModel;
import javax.swing.JButton;
import javax.swing.JComboBox;
import javax.swing.JComponent;
import javax.swing.JDialog;
import javax.swing.JLabel;
import javax.swing.JList;
import javax.swing.JOptionPane;
import javax.swing.JPanel;
import javax.swing.JScrollPane;
import javax.swing.KeyStroke;
import javax.swing.SwingUtilities;
import javax.swing.WindowConstants;
import javax.swing.event.ListSelectionEvent;
import javax.swing.event.ListSelectionListener;

import org.infinity.gui.ViewerUtil;
import org.infinity.resource.effects.BaseOpcode;
import org.infinity.util.Misc;

/**
 * Dialog for customizing blacklist of effect opcodes that is considered by calculation of level-scaled durations.
 */
public class BlackListDialog extends JDialog implements ActionListener, ListSelectionListener, MouseListener {
  private final Set<Integer> defaultOpcodes = new TreeSet<>();

  private JList<Opcode> blackList;
  private DefaultListModel<Opcode> blackListModel;
  private JButton bAdd, bRemove, bReset, bClose;

  public BlackListDialog(Window owner, Collection<Integer> opcodes) {
    super(owner, "Opcodes Blacklist", Dialog.ModalityType.DOCUMENT_MODAL);
    init();
    applyConfig(opcodes);
  }

  /** Returns the user-defined set of opcodes. */
  public Set<Integer> getConfig() {
    if (blackListModel != null) {
      final TreeSet<Integer> retVal = new TreeSet<>();
      for (int i = 0, count = blackListModel.size(); i < count; i++) {
        retVal.add(blackListModel.get(i).intValue());
      }
      return retVal;
    } else {
      return defaultOpcodes;
    }
  }

  /** Applies the specified set of opcodes to the dialog. Old values are discarded. */
  public void applyConfig(Collection<Integer> opcodes) {
    defaultOpcodes.clear();
    blackListModel.clear();
    if (opcodes != null) {
      defaultOpcodes.addAll(opcodes);
      for (final Integer opcode : opcodes) {
        blackListModel.addElement(new Opcode(opcode));
      }
    }
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent e) {
    if (e.getSource() == bAdd) {
      addItem();
    } else if (e.getSource() == bRemove) {
      removeItem(blackList.getSelectedIndex());
    } else if (e.getSource() == bReset) {
      resetList();
    } else if (e.getSource() == bClose) {
      close();
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  // --------------------- Begin Interface ListSelectionListener ---------------------

  @Override
  public void valueChanged(ListSelectionEvent e) {
    if (e.getSource() == blackList) {
      bRemove.setEnabled(blackList.getSelectedIndex() >= 0);
    }
  }

  // --------------------- End Interface ListSelectionListener ---------------------

  // --------------------- Begin Interface MouseListener ---------------------

  @Override
  public void mouseClicked(MouseEvent e) {
    if (e.getSource() == blackList) {
      // double-click on list adds new item
      if (e.getClickCount() == 2) {
        addItem();
      }
    }
  }

  @Override
  public void mousePressed(MouseEvent e) {
  }

  @Override
  public void mouseReleased(MouseEvent e) {
  }

  @Override
  public void mouseEntered(MouseEvent e) {
  }

  @Override
  public void mouseExited(MouseEvent e) {
  }

  // --------------------- End Interface MouseListener ---------------------

  /** Adds a new opcode to the list interactively. */
  private void addItem() {
    final DefaultComboBoxModel<Opcode> itemsModel = new DefaultComboBoxModel<>();
    final String[] names = BaseOpcode.getEffectNames();
    for (int i = 0; i < names.length; i++) {
      itemsModel.addElement(new Opcode(i));
    }
    final JComboBox<Opcode> items = new JComboBox<>(itemsModel);
    items.setEditable(false);

    final int selectedIndex = blackList.getSelectedIndex();
    if (selectedIndex >= 0 && selectedIndex < blackListModel.size()) {
      itemsModel.setSelectedItem(blackListModel.get(selectedIndex));
    }

    final int result = JOptionPane.showConfirmDialog(this, items, "Add Opcode", JOptionPane.OK_CANCEL_OPTION);
    if (result == JOptionPane.OK_OPTION) {
      final int idx = items.getSelectedIndex();
      if (idx >= 0) {
        final Opcode item = itemsModel.getElementAt(idx);
        addItem(item, true);
      }
    }
  }

  /** Adds the specified opcode to the list unless it exists already. */
  private void addItem(Opcode opcode, boolean select) {
    if (opcode != null) {
      // don't add duplicate items
      for (int i = 0, count = blackListModel.getSize(); i < count; i++) {
        if (blackListModel.get(i).intValue() == opcode.intValue()) {
          if (select) {
            blackList.setSelectedIndex(i);
            blackList.ensureIndexIsVisible(i);
          }
          return;
        }
      }

      blackListModel.addElement(opcode);
      if (select) {
        SwingUtilities.invokeLater(() -> {
          final int index = blackListModel.getSize() - 1;
          blackList.setSelectedIndex(index);
          blackList.ensureIndexIsVisible(index);
        });
      }
    }
  }

  /** Removes the opcode item at the specified list index. */
  private void removeItem(int selectedIndex) {
    if (selectedIndex >= 0 && selectedIndex < blackListModel.size()) {
      blackListModel.remove(selectedIndex);
      if (!blackListModel.isEmpty()) {
        blackList.setSelectedIndex(Math.min(selectedIndex, blackListModel.size() - 1));
      }
    }
  }

  /** Discards current list of opcodes adds default opcodes. */
  private void resetList() {
    blackListModel.clear();
    for (final Integer opcode : defaultOpcodes) {
      blackListModel.addElement(new Opcode(opcode));
    }
  }

  /** Closes the dialog. */
  private void close() {
    setVisible(false);
  }

  private void init() {
    bAdd = new JButton("Add...");
    bAdd.addActionListener(this);

    bRemove = new JButton("Remove");
    bRemove.addActionListener(this);

    bReset = new JButton("Reset");
    bReset.addActionListener(this);

    final JLabel lBlackList = new JLabel("Opcodes:");

    blackListModel = new DefaultListModel<>();
    blackList = new JList<>(blackListModel);
    blackList.addListSelectionListener(this);
    blackList.addMouseListener(this);
    final Dimension dimList = blackList.getPreferredSize();
    final Dimension dimItem = Misc.getTextDimension(blackList, "Use EFF when item equipped (do not use) (182)", true);
    dimList.width = dimItem.width;
    dimList.height = dimItem.height * 10;
    resetList();
    if (!blackListModel.isEmpty()) {
      blackList.setSelectedIndex(0);
    }

    final JScrollPane scroll = new JScrollPane(blackList);
    scroll.setPreferredSize(dimList);
    scroll.setHorizontalScrollBarPolicy(JScrollPane.HORIZONTAL_SCROLLBAR_AS_NEEDED);
    scroll.setVerticalScrollBarPolicy(JScrollPane.VERTICAL_SCROLLBAR_AS_NEEDED);

    bClose = new JButton("Close");
    bClose.addActionListener(this);

    final GridBagConstraints c = new GridBagConstraints();

    final JPanel panelTop = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    panelTop.add(lBlackList, c);

    final JPanel panelCenterButtons = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    panelCenterButtons.add(bAdd, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelCenterButtons.add(bRemove, c);
    ViewerUtil.setGBC(c, 0, 2, 1, 1, 1.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.BOTH,
        new Insets(8, 0, 0, 0), 0, 0);
    panelCenterButtons.add(new JPanel(), c);
    ViewerUtil.setGBC(c, 0, 3, 1, 1, 1.0, 0.0, GridBagConstraints.LAST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelCenterButtons.add(bReset, c);

    final JPanel panelCenter = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.BOTH,
        new Insets(0, 0, 0, 0), 0, 0);
    panelCenter.add(scroll, c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 0.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.VERTICAL,
        new Insets(0, 8, 0, 0), 0, 0);
    panelCenter.add(panelCenterButtons, c);

    final JPanel panelBottom = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    panelBottom.add(new JPanel(), c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 0.0, 0.0, GridBagConstraints.CENTER, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    panelBottom.add(bClose, c);
    ViewerUtil.setGBC(c, 0, 2, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    panelBottom.add(new JPanel(), c);

    final Container contentPane = getContentPane();
    setLayout(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 8, 0, 8), 0, 0);
    contentPane.add(panelTop, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 1.0, GridBagConstraints.LINE_START, GridBagConstraints.BOTH,
        new Insets(4, 8, 0, 8), 0, 0);
    contentPane.add(panelCenter, c);
    ViewerUtil.setGBC(c, 0, 2, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 8, 0, 8), 0, 0);
    contentPane.add(panelBottom, c);

    pack();
    setResizable(false);
    setLocationRelativeTo(getParent());

    // "Closing" the dialog only makes it invisible
    setDefaultCloseOperation(WindowConstants.HIDE_ON_CLOSE);

    getRootPane().setDefaultButton(bAdd);

    getRootPane().getInputMap(JComponent.WHEN_IN_FOCUSED_WINDOW).put(KeyStroke.getKeyStroke(KeyEvent.VK_ESCAPE, 0),
        getRootPane());
    getRootPane().getActionMap().put(getRootPane(), new AbstractAction() {
      @Override
      public void actionPerformed(ActionEvent event) {
        close();
      }
    });
  }
}

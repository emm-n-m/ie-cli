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
import java.awt.Rectangle;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.KeyEvent;
import java.awt.event.MouseEvent;
import java.awt.event.MouseListener;
import java.util.Collection;
import java.util.Collections;
import java.util.Set;
import java.util.TreeSet;

import javax.swing.AbstractAction;
import javax.swing.DefaultListModel;
import javax.swing.JButton;
import javax.swing.JComponent;
import javax.swing.JDialog;
import javax.swing.JLabel;
import javax.swing.JList;
import javax.swing.JPanel;
import javax.swing.JScrollPane;
import javax.swing.KeyStroke;
import javax.swing.SwingUtilities;
import javax.swing.WindowConstants;
import javax.swing.event.ListSelectionEvent;
import javax.swing.event.ListSelectionListener;

import org.infinity.gui.ViewerUtil;
import org.infinity.resource.Profile;
import org.infinity.util.Misc;

/**
 * Dialog that manages a list of specific effect customizations.
 */
public class EffectsListDialog extends JDialog implements ActionListener, ListSelectionListener, MouseListener {
  private final Set<EffectConfig> effects = new TreeSet<>();

  private JList<EffectConfig> effectsList;
  private DefaultListModel<EffectConfig> effectsListModel;
  private JButton bAdd, bEdit, bRemove, bClose;
  private EffectConfigDialog effectDialog;

  public EffectsListDialog(GeneratorDialog parent) {
    super(parent, "Advanced Configuration", Dialog.ModalityType.DOCUMENT_MODAL);
    init();
  }

  /** Returns the set of user-defined effect configurations. */
  public Set<EffectConfig> getConfig() {
    return Collections.unmodifiableSet(effects);
  }

  /** Defines a new set of effect configurations. Old set is discarded. */
  public void applyConfig(Collection<EffectConfig> newSet) {
    effects.clear();
    effectsListModel.clear();
    if (newSet != null) {
      effects.addAll(newSet);
      for (final EffectConfig cfg : newSet) {
        effectsListModel.addElement(cfg);
      }
    }
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent e) {
    if (e.getSource() == bAdd) {
      addItem(false);
    } else if (e.getSource() == bEdit) {
      addItem(true);
    } else if (e.getSource() == bRemove) {
      removeItem(effectsList.getSelectedIndex());
    } else if (e.getSource() == bClose) {
      setVisible(false);
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  // --------------------- Begin Interface ListSelectionListener ---------------------

  @Override
  public void valueChanged(ListSelectionEvent e) {
    if (e.getSource() == effectsList) {
      onListItemSelect(effectsList.getSelectedIndex());
    }
  }

  // --------------------- End Interface ListSelectionListener ---------------------

  // --------------------- Begin Interface MouseListener ---------------------

  @Override
  public void mouseClicked(MouseEvent e) {
    if (e.getSource() == effectsList) {
      // double-click on item opens edit dialog
      final int index = effectsList.getSelectedIndex();
      if (e.getClickCount() == 2 && index >= 0) {
        final Rectangle rect = effectsList.getCellBounds(index, index);
        if (rect != null && rect.contains(e.getPoint())) {
          addItem(true);
        }
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

  private void addItem(boolean editExisting) {
    EffectConfig config = null;

    if (editExisting) {
      final int selectedIndex = effectsList.getSelectedIndex();
      if (selectedIndex >= 0 && selectedIndex < effectsListModel.size()) {
        config = effectsListModel.get(selectedIndex);
      }
    }

    if (effectDialog == null) {
      effectDialog = new EffectConfigDialog(SwingUtilities.getWindowAncestor(this), config);
    } else {
      effectDialog.applyConfig(config);
    }

    effectDialog.setVisible(true);
    config = effectDialog.getConfig();
    if (config != null) {
      addItem(config, true);
    }
  }

  private void addItem(EffectConfig config, boolean select) {
    if (config != null) {
      if (effects.remove(config)) {
        // updating existing item
        effects.add(config);
        for (int i = 0, count = effectsListModel.size(); i < count; i++) {
          if (config.equals(effectsListModel.get(i))) {
            effectsListModel.set(i, config);
            if (select) {
              effectsList.setSelectedIndex(i);
            }
            break;
          }
        }
      } else {
        // adding new item (in sorted order)
        effects.add(config);
        boolean added = false;
        for (int i = 0, count = effectsListModel.size(); i < count; i++) {
          if (config.compareTo(effectsListModel.get(i)) < 0) {
            effectsListModel.add(i, config);
            if (select) {
              effectsList.setSelectedIndex(i);
            }
            added = true;
            break;
          }
        }

        if (!added) {
          effectsListModel.addElement(config);
          if (select) {
            effectsList.setSelectedIndex(ensureListIndex(effectsListModel.size() - 1));
          }
        }
      }
    }
  }

  /** Removes the specified list item. */
  private void removeItem(int index) {
    index = ensureListIndex(index);
    if (index >= 0) {
      effectsListModel.remove(index);
      if (!effectsListModel.isEmpty()) {
        effectsList.setSelectedIndex(ensureListIndex(index));
      }
    }
  }

  /** Updates UI controls states based on the specified list item selection. */
  private void onListItemSelect(int index) {
    bEdit.setEnabled(ensureListIndex(index) >= 0);
    bRemove.setEnabled(ensureListIndex(index) >= 0);
  }

  /***/
  private void reset() {
    effectsListModel.clear();
    for (final EffectConfig config : effects) {
      effectsListModel.addElement(config);
    }
    effectsList.setSelectedIndex(ensureListIndex(0));
    onListItemSelect(effectsList.getSelectedIndex());
  }

  /**
   * Makes sure that the specified list item index is within valid bounds. Returns -1 if the index is out of bounds.
   */
  private int ensureListIndex(int index) {
    return Math.max(-1, Math.min(effectsListModel.size() - 1, index));
  }

  private void init() {
    bClose = new JButton("Close");
    bClose.addActionListener(this);

    bAdd = new JButton("Add...");
    bAdd.addActionListener(this);

    bEdit = new JButton("Edit...");
    bEdit.addActionListener(this);

    bRemove = new JButton("Remove");
    bRemove.addActionListener(this);

    final JLabel lEffectsList = new JLabel("Effects:");

    effectsListModel = new DefaultListModel<>();
    effectsList = new JList<>(effectsListModel);
    effectsList.addListSelectionListener(this);
    effectsList.addMouseListener(this);

    // calculating default list dimension
    final Dimension dimList = effectsList.getPreferredSize();
    final EffectConfig config = new EffectConfig();
    config.setOpcode((Profile.getEngine() == Profile.Engine.PST) ? 156 : 182);
    config.getParameter1(0).setEnabled(true);
    final String prototype = config.toString();
    final Dimension dimItem = Misc.getTextDimension(effectsList, prototype, true);
    dimList.width = dimItem.width;
    dimList.height = dimItem.height * 10;

    reset();

    final JScrollPane scroll = new JScrollPane(effectsList);
    scroll.setPreferredSize(dimList);
    scroll.setHorizontalScrollBarPolicy(JScrollPane.HORIZONTAL_SCROLLBAR_AS_NEEDED);
    scroll.setVerticalScrollBarPolicy(JScrollPane.VERTICAL_SCROLLBAR_AS_NEEDED);

    final GridBagConstraints c = new GridBagConstraints();

    final JPanel panelTop = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    panelTop.add(lEffectsList, c);

    final JPanel panelCenterButtons = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    panelCenterButtons.add(bAdd, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelCenterButtons.add(bEdit, c);
    ViewerUtil.setGBC(c, 0, 2, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelCenterButtons.add(bRemove, c);
    ViewerUtil.setGBC(c, 0, 3, 1, 1, 1.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.BOTH,
        new Insets(8, 0, 0, 0), 0, 0);
    panelCenterButtons.add(new JPanel(), c);

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
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.LINE_START, GridBagConstraints.BOTH,
        new Insets(8, 8, 0, 8), 0, 0);
    contentPane.add(panelTop, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 1.0, GridBagConstraints.LINE_START, GridBagConstraints.BOTH,
        new Insets(8, 8, 0, 8), 0, 0);
    contentPane.add(panelCenter, c);
    ViewerUtil.setGBC(c, 0, 2, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 8, 0, 8), 0, 0);
    contentPane.add(panelBottom, c);

    pack();
    setResizable(false);
    setLocationRelativeTo(getParent());

    // "Closing" the dialog only makes it invisible
    setDefaultCloseOperation(WindowConstants.HIDE_ON_CLOSE);

    getRootPane().getInputMap(JComponent.WHEN_IN_FOCUSED_WINDOW).put(KeyStroke.getKeyStroke(KeyEvent.VK_ESCAPE, 0),
        getRootPane());
    getRootPane().getActionMap().put(getRootPane(), new AbstractAction() {
      @Override
      public void actionPerformed(ActionEvent event) {
        setVisible(false);
      }
    });
  }
}

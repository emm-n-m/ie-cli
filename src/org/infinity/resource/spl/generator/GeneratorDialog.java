// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.resource.spl.generator;

import java.awt.Container;
import java.awt.Dialog;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.KeyEvent;
import java.util.Objects;

import javax.swing.AbstractAction;
import javax.swing.JComponent;
import javax.swing.JDialog;
import javax.swing.KeyStroke;
import javax.swing.WindowConstants;

import org.infinity.datatype.IsNumeric;
import org.infinity.gui.ViewerUtil;
import org.infinity.resource.spl.SplResource;

/**
 * Dialog class for use with the {@link GeneratorFrame} frame.
 */
public class GeneratorDialog extends JDialog {
  private final GeneratorFrame frame;

  private boolean accepted;

  /** Removes all static instances referenced by this class to clear all input data. */
  public static void clearInstance() {
    GeneratorFrame.clearInstance();
  }

  /**
   * Opens a dialog for the user to define parameters for the ability generator and returns the configuration on
   * success.
   *
   * @param struct {@link SplResource} instance for which the abilities should be generated.
   * @return {@link GeneratorConfig} structure with parameters if accepted, {@code null} otherwise.
   * @throws Exception if an error occurred.
   */
  public static GeneratorConfig getConfiguration(SplResource struct) throws Exception {
    // SplResource must exist
    if (struct == null) {
      throw new NullPointerException("struct is null");
    }

    // At least one Ability structure must exist
    if (((IsNumeric)struct.getAttribute(SplResource.SPL_NUM_ABILITIES)).getValue() < 1) {
      throw new Exception("At least one ability structure must be defined.");
    }

    final GeneratorDialog dlg = new GeneratorDialog(struct);
    return dlg.getConfig();
  }

  private GeneratorDialog(SplResource struct) {
    super(ViewerUtil.getWindowAncestor(Objects.requireNonNull(struct).getViewer()), "Spell Abilities Generator",
        Dialog.ModalityType.DOCUMENT_MODAL);
    frame = GeneratorFrame.getInstance();
    frame.setDialog(this);
    frame.setStructure(struct);
    init();
    frame.applyConfig(frame.getConfig());
  }

  /**
   * Returns a fully initialized {@link GeneratorConfig} if the user invoked the "Generate" action. Otherwise,
   * {@code null} is returned.
   */
  public GeneratorConfig getConfig() {
    return accepted ? getFrame().getConfig() : null;
  }

  /** Called internally if the "Generate" action is invoked. */
  protected void accept() {
//    getFrame().updateConfig();
    accepted = true;
    setVisible(false);
  }

  /** Called internally if the "Cancel" action is invoked. */
  protected void cancel() {
    accepted = false;
    setVisible(false);
  }

  /** Returns the associated {@link GeneratorFrame} panel. */
  private GeneratorFrame getFrame() {
    return frame;
  }

  private void init() {
    final GridBagConstraints c = new GridBagConstraints();

    final Container contentPane = getContentPane();
    contentPane.setLayout(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.CENTER, GridBagConstraints.BOTH,
        new Insets(12, 12, 12, 12), 0, 0);
    contentPane.add(getFrame(), c);

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
        cancel();
      }
    });

    setVisible(true);
  }
}

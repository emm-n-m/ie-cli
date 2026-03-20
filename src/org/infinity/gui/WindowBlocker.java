// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui;

import java.awt.Component;
import java.awt.Cursor;
import java.awt.Window;
import java.awt.event.MouseAdapter;

import javax.swing.RootPaneContainer;

import org.infinity.NearInfinity;

/**
 * @author Jon Heggland
 */
public final class WindowBlocker {
  private static final MouseAdapter DUMMY_MOUSE_LISTENER = new MouseAdapter() {
  };

  private Component glassPane;

  /**
   * Blocks or unblocks the whole GUI and shows the respective mouse cursor.
   *
   * @param block Blocks the GUI if set to {@code true}, unblocks if set to {@code false}.
   */
  public static void blockWindow(boolean block) {
    blockWindow(NearInfinity.getInstance(), block);
  }

  /**
   * Blocks or unblocks the specified component and shows the respective mouse cursor.
   *
   * @param block Blocks the component if set to {@code true}, unblocks if set to {@code false}.
   */
  public static void blockWindow(Window window, boolean block) {
    if (window != null) {
      if (block) {
        window.setCursor(Cursor.getPredefinedCursor(Cursor.WAIT_CURSOR));
      } else {
        window.setCursor(null);
      }
    }
  }

  /**
   * Returns the first {@code RootPaneContainer} ancestor of the specified {@code Component}. Failing that it returns
   * the {@code defContainer} value.
   *
   * @param comp         {@link Component} to get {@link RootPaneContainer} ancestor of.
   * @param defContainer Default {@link RootPaneContainer} to return if {@code comp} doesn't yield a non-{@code null}
   *                       result.
   * @return the first {@link RootPaneContainer} ancestor of {@code comp}, or {@code defContainer} if {@comp} is not
   *         contained inside a {@code RootPaneContainer}.
   */
  public static RootPaneContainer getRootPaneAncestor(Component comp, RootPaneContainer defContainer) {
    RootPaneContainer retVal = null;

    while (comp != null) {
      if (comp instanceof RootPaneContainer) {
        retVal = (RootPaneContainer)comp;
        break;
      }
      comp = comp.getParent();
    }

    if (retVal == null) {
      retVal = defContainer;
    }

    return retVal;
  }

  public WindowBlocker(RootPaneContainer window) {
    if (window == null) {
      return;
    }
    glassPane = window.getGlassPane();
    glassPane.setCursor(Cursor.getPredefinedCursor(Cursor.WAIT_CURSOR));
    glassPane.addMouseListener(DUMMY_MOUSE_LISTENER);
  }

  public void setBlocked(boolean blocked) {
    if (glassPane == null) {
      return;
    }
    if (blocked != glassPane.isVisible()) {
      glassPane.setVisible(blocked);
    }
  }
}

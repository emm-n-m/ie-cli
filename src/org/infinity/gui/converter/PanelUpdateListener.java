// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter;

import java.util.EventListener;

/** The listener for receiving events about panel updates. */
public interface PanelUpdateListener extends EventListener {
  /** Invoked when a panel update happens. */
  void panelUpdated(PanelUpdateEvent e);
}

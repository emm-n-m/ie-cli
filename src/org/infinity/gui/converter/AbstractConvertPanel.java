// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter;

import java.awt.Component;
import java.awt.LayoutManager;
import java.io.File;
import java.nio.file.Path;
import java.util.Collection;

import javax.swing.BorderFactory;
import javax.swing.JFileChooser;
import javax.swing.JPanel;
import javax.swing.event.ChangeListener;
import javax.swing.event.EventListenerList;
import javax.swing.filechooser.FileNameExtensionFilter;

import org.infinity.gui.ViewerUtil;
import org.infinity.resource.Profile;
import org.infinity.util.io.FileEx;
import org.infinity.util.io.FileManager;

/**
 * Common base class for converter panels which implements change listener functionality.
 */
public abstract class AbstractConvertPanel extends JPanel {
  protected final EventListenerList listenerList = new EventListenerList();

  /**
   * Shows an "Select File(s)" dialog and returns one or more selected filenames.
   *
   * @param parent      Parent {@link Component} for the file dialog.
   * @param title       Title of the file dialog.
   * @param initialPath Initial {@link Path} that is opened in the file dialog. Can point to folder or file.
   * @param multiSelect Specifies whether multiple files can be selected in the file dialog.
   * @param filters     A list of file extension files to use.
   * @param filterIndex File extension filter index to set.
   * @return Array of selected file {@link Path}s. Returns {@code null} of file dialog was cancelled.
   */
  public static Path[] getOpenFileNames(Component parent, String title, Path initialPath, boolean multiSelect,
      Collection<FileNameExtensionFilter> filters, int filterIndex) {
    if (initialPath == null) {
      initialPath = Profile.getGameRoot();
    }
    if (title == null) {
      title = multiSelect ? "Select File(s)" : "Select File";
    }

    final Path file = FileManager.resolve(initialPath);
    final JFileChooser fc = new JFileChooser(file.toFile());
    if (!FileEx.create(file).isDirectory()) {
      fc.setSelectedFile(file.toFile());
    }
    fc.setDialogTitle(title);
    fc.setDialogType(JFileChooser.OPEN_DIALOG);
    fc.setFileSelectionMode(JFileChooser.FILES_ONLY);
    fc.setMultiSelectionEnabled(multiSelect);

    if (filters != null) {
      FileNameExtensionFilter selectedFilter = null;
      int idx = 0;
      for (final FileNameExtensionFilter filter : filters) {
        fc.addChoosableFileFilter(filter);
        if (idx == filterIndex) {
          selectedFilter = filter;
        }
        idx++;
      }
      if (selectedFilter != null) {
        fc.setFileFilter(selectedFilter);
      }
    }

    final int result = fc.showOpenDialog(ViewerUtil.getWindowAncestor(parent));
    if (result == JFileChooser.APPROVE_OPTION) {
      final Path[] retVal;
      if (fc.isMultiSelectionEnabled()) {
        final File[] files = fc.getSelectedFiles();
        retVal = new Path[files.length];
        for (int i = 0; i < files.length; i++) {
          retVal[i] = files[i].toPath();
        }
      } else {
        retVal = new Path[1];
        retVal[0] = fc.getSelectedFile().toPath();
      }
      return retVal;
    }

    return null;
  }

  /**
   * Shows an "Select Folder" dialog and returns the selected directory path.
   *
   * @param parent      Parent {@link Component} for the file dialog.
   * @param title       Title of the file dialog.
   * @param initialPath Initial {@link Path} that is opened in the file dialog.
   * @return Directory as {@link Path} instance of successful, {@code null} otherwise.
   */
  public static Path getOpenDirectory(Component parent, String title, Path initialPath) {
    if (initialPath == null) {
      initialPath = Profile.getGameRoot();
    }
    if (title == null) {
      title = "Select Folder";
    }

    final JFileChooser fc = new JFileChooser(initialPath.toFile());
    fc.setDialogTitle(title);
    fc.setDialogType(JFileChooser.OPEN_DIALOG);
    fc.setFileSelectionMode(JFileChooser.DIRECTORIES_ONLY);

    final int result = fc.showOpenDialog(ViewerUtil.getWindowAncestor(parent));
    if (result == JFileChooser.APPROVE_OPTION) {
      final Path retVal = fc.getSelectedFile().toPath();
      return retVal;
    }

    return null;
  }

  /**
   * Initializes a JPanel with {@link ChangeListener} support.
   *
   * @param layout The LayoutManager to use.
   */
  protected AbstractConvertPanel(LayoutManager layout) {
    super(layout);
  }

  /**
   * Initializes a JPanel with a titled border and {@link ChangeListener} support.
   *
   * @param layout The LayoutManager to use.
   * @param title  Title string that is used for the border around the panel.
   */
  protected AbstractConvertPanel(LayoutManager layout, String title) {
    super(layout);

    String borderTitle = (title != null) ? title : "";
    if (!borderTitle.isEmpty()) {
      borderTitle += ' ';
    }
    setBorder(BorderFactory.createTitledBorder(borderTitle));
  }

  /**
   * Adds a listener to the list that's notified each time a panel update occurs.
   *
   * @param l the {@link PanelUpdateListener} to be added
   */
  public void addPanelUpdateListener(PanelUpdateListener l) {
    if (l != null) {
      listenerList.add(PanelUpdateListener.class, l);
    }
  }

  /**
   * Removes a listener from the list that's notified each time a panel update occurs.
   *
   * @param l the {@link ChangeListener} to be removed
   */
  public void removePanelUpdateListener(PanelUpdateListener l) {
    if (l != null) {
      listenerList.remove(PanelUpdateListener.class, l);
    }
  }

  /**
   * Returns an array of all the panel update listeners registered to this converter panel.
   *
   * @return all of the panel's {@link PanelUpdateListener}s, or an empty array if no listeners are currently registered
   */
  public PanelUpdateListener[] getPanelUpdateListeners() {
    return listenerList.getListeners(PanelUpdateListener.class);
  }

  /**
   * Sends a {@link PanelUpdateEvent} to all registered {@link PanelUpdateListener}s.
   *
   * @param reason A {@link UpdateReason} constant that provides more details about the panel update.
   */
  protected void firePanelUpdate(UpdateReason reason) {
    firePanelUpdate(reason, null);
  }

  /**
   * Sends a {@link PanelUpdateEvent} to all registered {@link PanelUpdateListener}s.
   *
   * @param reason A {@link UpdateReason} constant that provides more details about the panel update.
   * @param params Optional list of parameters associated with the {@code reason}.
   */
  protected void firePanelUpdate(UpdateReason reason, Collection<Object> params) {
    final Object[] listeners = listenerList.getListenerList();
    PanelUpdateEvent ev = null;

    for (int i = listeners.length - 2; i >= 0; i -= 2) {
      if (listeners[i] == PanelUpdateListener.class) {
        if (ev == null) {
          ev = new PanelUpdateEvent(this, reason, params);
        }
        ((PanelUpdateListener)listeners[i + 1]).panelUpdated(ev);
      }
    }
  }
}

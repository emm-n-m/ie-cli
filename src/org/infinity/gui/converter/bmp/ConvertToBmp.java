// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bmp;

import java.awt.Dimension;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Image;
import java.awt.Insets;
import java.awt.image.BufferedImage;
import java.awt.image.DataBufferInt;
import java.io.OutputStream;
import java.nio.ByteBuffer;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;

import javax.swing.JOptionPane;
import javax.swing.JPanel;

import org.infinity.gui.ChildFrame;
import org.infinity.gui.ViewerUtil;
import org.infinity.gui.converter.ConvertButtonsPanel;
import org.infinity.gui.converter.ConvertFileListPanel;
import org.infinity.gui.converter.ConvertOutputPanel;
import org.infinity.gui.converter.ConvertOutputPanel.Overwrite;
import org.infinity.gui.converter.PanelUpdateEvent;
import org.infinity.gui.converter.PanelUpdateListener;
import org.infinity.icon.Icons;
import org.infinity.resource.Profile;
import org.infinity.resource.graphics.ColorConvert;
import org.infinity.util.io.FileManager;
import org.infinity.util.io.StreamUtils;
import org.tinylog.Logger;

/**
 * Dialog for converting graphics files to the BMP format, including variants that are not natively supported by Java.
 */
public class ConvertToBmp extends ChildFrame implements PanelUpdateListener {
  private static Path currentPath = Profile.getGameRoot();

  private ConvertFileListPanel fileListPanel;
  private ConvertOutputPanel outputPanel;
  private BmpOptionsPanel optionsPanel;
  private ConvertButtonsPanel buttonsPanel;

  public ConvertToBmp() {
    super("Convert to BMP", true);
    init();
  }

  // --------------------- Begin Class ChildFrame ---------------------

  /** Resets dialog content and optionally hides the dialog window. */
  public void hideWindow(boolean hide) {
    reset();
    if (hide) {
      setVisible(false);
    }
  }

  @Override
  protected boolean windowClosing(boolean forced) throws Exception {
    currentPath = outputPanel.getCurrentDirectory();
    reset();
    return super.windowClosing(forced);
  }

  // --------------------- End Class ChildFrame ---------------------

  // --------------------- Begin Interface PanelUpdateListener ---------------------

  @Override
  public void panelUpdated(PanelUpdateEvent e) {
    if (e.getSource() == fileListPanel) {
      handleFileListPanel(e);
    } else if (e.getSource() == outputPanel) {
      handleOutputPanel(e);
    } else if (e.getSource() == buttonsPanel) {
      handleButtonPanel(e);
    }
  }

  // --------------------- End Interface PanelUpdateListener ---------------------

  /** Handles state changes in the file list panel. */
  private void handleFileListPanel(PanelUpdateEvent e) {
    updateStatus();
  }

  /** Handles state changes in the output panel. */
  private void handleOutputPanel(PanelUpdateEvent e) {
    updateStatus();
  }

  /** Handles state changes in the button panel. */
  private void handleButtonPanel(PanelUpdateEvent e) {
    if (e.getReason() == ConvertButtonsPanel.REASON_CONVERT) {
      convert();
    } else if (e.getReason() == ConvertButtonsPanel.REASON_CANCEL) {
      hideWindow(true);
    }
  }

  /** Resets session-specific dialog controls. */
  private void reset() {
    fileListPanel.reset();
    outputPanel.reset();
    optionsPanel.reset();
  }

  /** Updates the dialog control state. */
  private void updateStatus() {
    final int inputFileCount = fileListPanel.getFileCount();
    final boolean outputPathSet = !outputPanel.getOutputFolder().isEmpty();
    buttonsPanel.setConvertButtonEnabled(inputFileCount > 0 && outputPathSet);
  }

  /** Performs the file conversion. */
  private void convert() {
    final List<Path> inFiles = fileListPanel.getFiles();
    if (inFiles.isEmpty()) {
      JOptionPane.showMessageDialog(this, "No input files defined.", "Error", JOptionPane.ERROR_MESSAGE);
      return;
    }

    Path outPath = null;
    try {
      outPath = FileManager.resolve(outputPanel.getOutputFolder());
    } catch (Exception e) {
      Logger.debug(e);
    }

    if (outPath == null) {
      JOptionPane.showMessageDialog(this, "Invalid output directory specified.", "Error", JOptionPane.ERROR_MESSAGE);
      return;
    }

    final Overwrite mode = outputPanel.getOverwriteMode();
    final boolean hasAlpha = optionsPanel.isAlphaSelected();
    final boolean premultipliedAlpha = optionsPanel.isPremultipliedAlphaFixSelected();
    final boolean closeOnExit = buttonsPanel.isCloseOnExit();

    try {
      final BmpWorker worker = new BmpWorker(this, inFiles, outPath, mode, hasAlpha, premultipliedAlpha, closeOnExit);
      worker.execute();
    } catch (Exception e) {
      Logger.error(e);
      final String msg;
      if (e.getMessage() == null || e.getMessage().isEmpty()) {
        msg = "Error: " + e.getMessage();
      } else {
        msg = e.getClass().getSimpleName() + " error occurred.";
      }
      JOptionPane.showMessageDialog(this, msg, "Error", JOptionPane.ERROR_MESSAGE);
    }
  }

  private void init() {
    setIconImage(Icons.ICON_APPLICATION_16.getIcon().getImage());

    fileListPanel = new ConvertFileListPanel("Input", currentPath, ColorConvert.GRAPHICS_FILTERS_DEFAULT);
    fileListPanel.addPanelUpdateListener(this);

    optionsPanel = new BmpOptionsPanel();

    outputPanel = new ConvertOutputPanel("Output", currentPath, optionsPanel);
    outputPanel.addPanelUpdateListener(this);

    buttonsPanel = new ConvertButtonsPanel();
    buttonsPanel.addPanelUpdateListener(this);

    final GridBagConstraints c = new GridBagConstraints();

    final JPanel panelMain = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.BOTH,
        new Insets(0, 0, 0, 0), 0, 0);
    panelMain.add(fileListPanel, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelMain.add(outputPanel, c);
    ViewerUtil.setGBC(c, 0, 2, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelMain.add(buttonsPanel, c);

    setLayout(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.CENTER, GridBagConstraints.BOTH,
        new Insets(8, 8, 8, 8), 0, 0);
    add(panelMain, c);

    updateStatus();

    setPreferredSize(new Dimension(getPreferredSize().width + 50, getPreferredSize().height + 50));
    setMinimumSize(getPreferredSize());
    pack();
    setLocationRelativeTo(getParent());
    setVisible(true);
  }

  /**
   * Creates a 32-bit BMP file that is compatible with EE games.
   *
   * @param srcImage             The source {@link Image} object.
   * @param outFile              Output file {@link Path}.
   * @param hasAlpha             Whether to enable alpha channel support in the output BMP.
   * @param isPremultipliedAlpha Whether source alpha is premultiplied.
   * @throws Exception if graphics conversion failed.
   */
  public static void writeBitmap(Image srcImage, Path outFile, boolean hasAlpha, boolean isPremultipliedAlpha)
      throws Exception {
    if (srcImage == null) {
      throw new NullPointerException("Source image is null");
    }
    if (outFile == null) {
      throw new NullPointerException("Output file path is null");
    }

    final BufferedImage image = ColorConvert.toBufferedImage(srcImage, true);
    final int[] pixels = ((DataBufferInt)image.getRaster().getDataBuffer()).getData();

    // "fixing" premultiplied alpha format
    if (hasAlpha && isPremultipliedAlpha) {
      if (pixels != null) {
        for (int i = 0; i < pixels.length; i++) {
          float anorm = ((pixels[i] >>> 24) & 0xff) / 255.0f;
          anorm = (anorm > 0.0f) ? 1.0f / anorm : 0.0f;
          int r = (int)((((pixels[i] >>> 16) & 0xff) * anorm) + 0.5f);
          if (r > 255) {
            r = 255;
          }
          int g = (int)((((pixels[i] >>> 8) & 0xff) * anorm) + 0.5f);
          if (g > 255) {
            g = 255;
          }
          int b = (int)(((pixels[i] & 0xff) * anorm) + 0.5f);
          if (b > 255) {
            b = 255;
          }
          pixels[i] = (pixels[i] & 0xff000000) | (r << 16) | (g << 8) | b;
        }
      }
    }

    final int bpp = hasAlpha ? 32 : 24;
    final int bytesPerPixel = bpp / 8;
    final int bytesPerLine = image.getWidth() * bytesPerPixel;
    final int fillBytes = (4 - (bytesPerLine & 3)) & 3;

    // writing BMP header
    final int compression = hasAlpha ? 3 : 0;
    final int sizeFileHeader = 14;
    final int sizeBitmapHeader = hasAlpha ? 124 : 40;
    final int headerSize = sizeFileHeader + sizeBitmapHeader;
    final int fileSize = headerSize + (bytesPerLine + fillBytes) * image.getHeight();
    final ByteBuffer buffer = StreamUtils.getByteBuffer(headerSize);

    // file header
    buffer.put("BM".getBytes()); // File type ("BM")
    buffer.putInt(fileSize); // total file size
    buffer.putInt(0); // reserved
    buffer.putInt(sizeFileHeader + sizeBitmapHeader); // start of pixel data
    // bitmap header
    buffer.putInt(sizeBitmapHeader); // bitmap header size
    buffer.putInt(image.getWidth()); // image width
    buffer.putInt(image.getHeight()); // image height
    buffer.putShort((short)1); // color planes
    buffer.putShort((short)bpp); // bits per pixel
    buffer.putInt(compression); // compression (0=uncompressed, 3=bitfield)
    buffer.putInt(image.getWidth() * image.getHeight() * 4); // size of bitmap in bytes
    buffer.putInt(0xb12); // pixels per meter
    buffer.putInt(0xb12); // pixels per meter
    buffer.putInt(0); // colors used (palette only)
    buffer.putInt(0); // important colors (palette only)

    if (hasAlpha) {
      buffer.putInt(0x00ff0000); // red bitmask
      buffer.putInt(0x0000ff00); // green bitmask
      buffer.putInt(0x000000ff); // blue bitmask
      buffer.putInt(0xff000000); // alpha bitmask
      buffer.put("BGRs".getBytes()); // color space type
      final byte[] zero = new byte[16 * 4];
      Arrays.fill(zero, (byte)0);
      buffer.put(zero); // remaining fields are empty
    }

    // writing BMP pixel data in ARGB format (upside down)
    try (final OutputStream os = StreamUtils.getOutputStream(outFile, true)) {
      // writing header
      os.write(buffer.array());

      // writing pixel data
      final int transThreshold = 0x20;
      final byte[] row = new byte[bytesPerLine + fillBytes];
      for (int y = image.getHeight() - 1; y >= 0; y--) {
        for (int i = 0, idx = y * image.getWidth(); i < bytesPerLine; i += bytesPerPixel, idx++) {
          if (!hasAlpha && (pixels[idx] >>> 24) < transThreshold) {
            pixels[idx] = 0x00ff00; // transparent pixels are translated into RGB(0, 255, 0)
          }
          row[i]     = (byte)(pixels[idx] & 0xff);
          row[i + 1] = (byte)((pixels[idx] >>> 8) & 0xff);
          row[i + 2] = (byte)((pixels[idx] >>> 16) & 0xff);
          if (hasAlpha) {
            row[i + 3] = (byte)((pixels[idx] >>> 24) & 0xff);
          }
        }
        // adding alignment bytes
        for (int i = 0; i < fillBytes; i++) {
          row[bytesPerLine + i] = (byte)0;
        }
        os.write(row);
      }
    }
  }

  // -------------------------- INNER CLASSES --------------------------

}

// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bam;

import java.awt.CardLayout;
import java.awt.Color;
import java.awt.Dimension;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.ItemEvent;
import java.awt.event.ItemListener;
import java.awt.image.BufferedImage;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;

import javax.swing.BorderFactory;
import javax.swing.JButton;
import javax.swing.JComboBox;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.SwingUtilities;
import javax.swing.event.ChangeEvent;
import javax.swing.event.ChangeListener;

import org.infinity.gui.ColorGrid;
import org.infinity.gui.ColorGrid.MouseOverEvent;
import org.infinity.gui.ColorGrid.MouseOverListener;
import org.infinity.gui.ViewerUtil;
import org.infinity.resource.graphics.ColorConvert;
import org.infinity.util.Misc;
import org.infinity.util.tuples.Triple;

/**
 * The base class for filters that manipulate on color/pixel level.
 */
public abstract class BamFilterBaseColor extends BamFilterBase {
  protected BamFilterBaseColor(ConvertToBam parent, String name, String desc) {
    super(parent, name, desc, Type.COLOR);
  }

  /**
   * Applies the filter to the specified BufferedImage object. The returned BufferedImage object can either ne the
   * modified source image or a new copy.
   *
   * @param frame The BufferedImage object to modify.
   * @return The resulting BufferedImage object.
   */
  public abstract BufferedImage process(BufferedImage frame) throws Exception;

  /** Parses a list of palette indices from a parameter string of the format "[idx1,idx2,...]". */
  protected int[] decodeColorList(String param) {
    int[] indices = null;
    if (param != null && param.matches("\\[.*\\]")) {
      String colorString = param.substring(1, param.length() - 1).trim();
      if (!colorString.isEmpty()) {
        String[] colors = colorString.split(",");
        indices = new int[colors.length];
        for (int i = 0; i < colors.length; i++) {
          indices[i] = Misc.toNumber(colors[i], -1);
          if (indices[i] < 0 || indices[i] > 255) {
            indices = null;
            break;
          }
        }
      } else {
        indices = new int[0];
      }
    }
    return indices;
  }

  /** Converts a list of palette indices into a parameter string. */
  protected String encodeColorList(int[] indices) {
    StringBuilder sb = new StringBuilder();
    sb.append('[');
    if (indices != null) {
      for (int i = 0; i < indices.length; i++) {
        if (i > 0) {
          sb.append(',');
        }
        sb.append(indices[i]);
      }
    }
    sb.append(']');
    return sb.toString();
  }

  // -------------------------- INNER CLASSES --------------------------

  /** Enumeration of available color sorting algorithms. */
  public enum ColorSort {
    /** Preserves original palette order. */
    ORIGINAL("Original Palette"),
    /** Sorts by hue (HSL colorspace H). */
    HSL_HUE("Hue"),
    /** Sorts by saturation (HSL colorspace S). */
    HSL_SATURATION("Saturation"),
    /** Sorts by perceptual lightness (CIELAB colorspace L). */
    LAB_LIGHTNESS("Perceptual Lightness"),
    /** Sorts by perceptual hue (green-red) (CIELAB colorspace a). */
    LAB_HUE_A("Perceptual Hue (green-red)"),
    /** Sorts by perceptual hue (green-red) (CIELAB colorspace b). */
    LAB_HUE_B("Perceptual Hue (blue-yellow)"),
    /** Sorts by red color component intensity (ARGB colorspace R). */
    ARGB_RED("Red"),
    /** Sorts by green color component intensity (ARGB colorspace G). */
    ARGB_GREEN("Green"),
    /** Sorts by blue color component intensity (ARGB colorspace B). */
    ARGB_BLUE("Blue"),
    /** Sorts by alpha intensity (ARGB colorspace A). Does nothing if palette does not contain alpha-blended colors. */
    ARGB_ALPHA("Alpha"),
    ;

    /** Retains original order. */
    private static final Comparator<Color> CMP_NO_CHANGE = (c1, c2) -> 0;

    /** Compares two {@link Color} entries by HSL hue. */
    private static final Comparator<Color> CMP_HSL_H = (c1, c2) -> {
      final Triple<Double, Double, Double> hsl1 = ColorConvert.convertRGBtoHSL(c1.getRGB(), true);
      final Triple<Double, Double, Double> hsl2 = ColorConvert.convertRGBtoHSL(c2.getRGB(), true);
      double diff = hsl1.getValue0() - hsl2.getValue0();
      if (diff == 0.0) {
        diff = hsl1.getValue2() - hsl2.getValue2();
      }
      return (diff < 0.0) ? -1 : ((diff > 0.0) ? 1 : 0);
    };

    /** Compares two {@link Color} entries by HSL saturation. */
    private static final Comparator<Color> CMP_HSL_S = (c1, c2) -> {
      final Triple<Double, Double, Double> hsl1 = ColorConvert.convertRGBtoHSL(c1.getRGB(), true);
      final Triple<Double, Double, Double> hsl2 = ColorConvert.convertRGBtoHSL(c2.getRGB(), true);
      double diff = hsl1.getValue1() - hsl2.getValue1();
      if (diff == 0.0) {
        diff = hsl1.getValue2() - hsl2.getValue2();
      }
      return (diff < 0.0) ? -1 : ((diff > 0.0) ? 1 : 0);
    };

    /** Compares two {@link Color} entries by CIELAB L. */
    private static final Comparator<Color> CMP_LAB_L = (c1, c2) -> {
      final Triple<Double, Double, Double> lab1 = ColorConvert.convertRGBtoLab(c1.getRGB());
      final Triple<Double, Double, Double> lab2 = ColorConvert.convertRGBtoLab(c2.getRGB());
      final double diff = lab1.getValue0() - lab2.getValue0();
      return (diff < 0.0) ? -1 : ((diff > 0.0) ? 1 : 0);
    };

    /** Compares two {@link Color} entries by CIELAB a. */
    private static final Comparator<Color> CMP_LAB_A = (c1, c2) -> {
      final Triple<Double, Double, Double> lab1 = ColorConvert.convertRGBtoLab(c1.getRGB());
      final Triple<Double, Double, Double> lab2 = ColorConvert.convertRGBtoLab(c2.getRGB());
      final double diff = lab1.getValue1() - lab2.getValue1();
      return (diff < 0.0) ? -1 : ((diff > 0.0) ? 1 : 0);
    };

    /** Compares two {@link Color} entries by CIELAB b. */
    private static final Comparator<Color> CMP_LAB_B = (c1, c2) -> {
      final Triple<Double, Double, Double> lab1 = ColorConvert.convertRGBtoLab(c1.getRGB());
      final Triple<Double, Double, Double> lab2 = ColorConvert.convertRGBtoLab(c2.getRGB());
      final double diff = lab1.getValue2() - lab2.getValue2();
      return (diff < 0.0) ? -1 : ((diff > 0.0) ? 1 : 0);
    };

    /** Compares two {@link Color} entries by RGB red. */
    private static final Comparator<Color> CMP_RGB_R = (c1, c2) -> {
      final int v1 = c1.getRed() * c1.getAlpha() / 255;
      final int v2 = c2.getRed() * c2.getAlpha() / 255;
      final int diff = v1 - v2;
      return (diff < 0) ? -1 : ((diff > 0) ? 1 : 0);
    };

    /** Compares two {@link Color} entries by RGB green. */
    private static final Comparator<Color> CMP_RGB_G = (c1, c2) -> {
      final int v1 = c1.getGreen() * c1.getAlpha() / 255;
      final int v2 = c2.getGreen() * c2.getAlpha() / 255;
      final int diff = v1 - v2;
      return (diff < 0) ? -1 : ((diff > 0) ? 1 : 0);
    };

    /** Compares two {@link Color} entries by RGB blue. */
    private static final Comparator<Color> CMP_RGB_B = (c1, c2) -> {
      final int v1 = c1.getBlue() * c1.getAlpha() / 255;
      final int v2 = c2.getBlue() * c2.getAlpha() / 255;
      final int diff = v1 - v2;
      return (diff < 0) ? -1 : ((diff > 0) ? 1 : 0);
    };

    /** Compares two {@link Color} entries by ARGB alpha. */
    private static final Comparator<Color> CMP_RGB_A = (c1, c2) -> {
      final int diff = c1.getAlpha() - c2.getAlpha();
      return (diff < 0) ? -1 : ((diff > 0) ? 1 : 0);
    };

    private static final List<Comparator<Color>> COMPARATORS = Arrays.asList(CMP_NO_CHANGE, CMP_HSL_H, CMP_HSL_S,
        CMP_LAB_L, CMP_LAB_A, CMP_LAB_B, CMP_RGB_R, CMP_RGB_G, CMP_RGB_B, CMP_RGB_A);

    private final String label;

    ColorSort(String label) {
      this.label = label;
    }

    @Override
    public String toString() {
      return label;
    }

    /** Returns the {@link Comparator} instance associated with the */
    public Comparator<Color> getComparator() {
      return COMPARATORS.get(ordinal());
    }
  }

  /**
   * Lets you select multiple color entries from the palette. It can be used to exclude the selected colors from
   * filtering.
   */
  public static class ExcludeColorsPanel extends JPanel implements MouseOverListener, ActionListener, ItemListener {
    private static final String FMT_INFO_RGB = "%d  %d  %d  %d";
    private static final String FMT_INFO_HEX_RGB = "#%02X%02X%02X%02X";

    private static final String CARD_MAIN = "MainPanel";
    private static final String CARD_MESSAGE = "MessagePanel";

    /** Storage of a global color selection. */
    private static int[] colorSelection = null;

    private final List<ChangeListener> listChangeListeners = new ArrayList<>();

    /** Lookup: Original color index -> Sorted color index */
    private final int[] colorIndexLookup = new int[256];

    /** Lookup: Sorted color index -> Original color index */
    private final int[] colorIndexReverseLookup = new int[256];

    private ColorGrid cgPalette;
    private JLabel lInfoIndex, lInfoRGB, lInfoHexRGB;
    private JButton bSelectAll, bSelectNone, bSelectInvert, bStoreSelect, bRestoreSelect;
    private JLabel messageLabel;
    private JButton messageButton;
    private JComboBox<ColorSort> cbColorSort;

    public ExcludeColorsPanel(int[] palette) {
      super(new CardLayout());
      init();
      updatePalette(palette);
    }

    /**
     * Adds a ChangeListener to the listener list. ChangeListeners will be notified whenever the color selection
     * changes.
     */
    public void addChangeListener(ChangeListener l) {
      if (l != null) {
        if (!listChangeListeners.contains(l)) {
          listChangeListeners.add(l);
        }
      }
    }

    /** Returns an array of all ChangeListeners added to this object. */
    public ChangeListener[] getChangeListeners() {
      ChangeListener[] retVal = new ChangeListener[listChangeListeners.size()];
      for (int i = 0; i < listChangeListeners.size(); i++) {
        retVal[i] = listChangeListeners.get(i);
      }
      return retVal;
    }

    /** Removes a ChangeListener from the listener list. */
    public void removeChangeListener(ChangeListener l) {
      if (l != null) {
        int idx = listChangeListeners.indexOf(l);
        if (idx >= 0) {
          listChangeListeners.remove(idx);
        }
      }
    }

    /** Applies the specified palette to the color grid component. */
    public void updatePalette(int[] palette) {
      final Color[] sortedPalette = getSortedPalette(getSelectedColorSort(), palette, colorIndexLookup,
          colorIndexReverseLookup);
      cgPalette.setColor(0, sortedPalette);
    }

    /** Returns the current palette from the color grid component. */
    public int[] getPalette() {
      final int colorCount = cgPalette.getColorCount();
      final int[] colors = new int[colorCount];
      for (int i = 0; i < colorCount; i++) {
        final Color color = cgPalette.getColor(colorIndexLookup[i]);
        colors[i] = color.getRGB();
      }
      return colors;
    }

    /** Returns the selected color indices as an array of integers. */
    public int[] getSelectedIndices() {
      final int[] sorted = cgPalette.getSelectedIndices();
      final int[] original = new int[sorted.length];
      for (int i = 0; i < original.length; i++) {
        original[i] = colorIndexReverseLookup[sorted[i]];
      }
      return original;
    }

    /** Returns whether the specified color index has been selected. */
    public boolean isSelectedIndex(int index) {
      if (index >= 0 && index < colorIndexLookup.length) {
        return cgPalette.isSelectedIndex(colorIndexLookup[index]);
      }
      return false;
    }

    /** Selects the specified color indices. Previous selections will be cleared. */
    public void setSelectedIndices(int[] indices) {
      cgPalette.clearSelection();
      if (indices != null) {
        final int[] sorted = new int[indices.length];
        for (int i = 0; i < sorted.length; i++) {
          sorted[i] = colorIndexLookup[indices[i]];
        }
        cgPalette.setSelectedIndices(sorted);
      }
    }

    /** Returns the currently selected {@link ColorSort} method. */
    public ColorSort getSelectedColorSort() {
      return cbColorSort.getModel().getElementAt(cbColorSort.getSelectedIndex());
    }

    /** Selects the specified {@link ColorSort} method. */
    public void setSelectedColorSort(ColorSort sort) {
      if (sort != null) {
        cbColorSort.setSelectedItem(sort);
      }
    }

    // --------------------- Begin Interface MouseOverListener ---------------------

    @Override
    public void mouseOver(MouseOverEvent event) {
      if (event.getSource() == cgPalette) {
        updateInfoBox(event.getColorIndex());
      }
    }

    // --------------------- End Interface MouseOverListener ---------------------

    // --------------------- Begin Interface ActionListener ---------------------

    @Override
    public void actionPerformed(ActionEvent event) {
      if (event.getSource() == cgPalette) {
        fireChangeListener();
      } else if (event.getSource() == bSelectAll) {
        cgPalette.setSelectedIndices(colorIndexLookup);
        fireChangeListener();
      } else if (event.getSource() == bSelectNone) {
        cgPalette.clearSelection();
        fireChangeListener();
      } else if (event.getSource() == bSelectInvert) {
        int[] selectedIndices = cgPalette.getSelectedIndices();
        int[] indices = new int[cgPalette.getColorCount() - selectedIndices.length];
        for (int i = 0, ofs = 0; i < cgPalette.getColorCount(); i++) {
          int idx = -1;
          for (int j = 0; j < selectedIndices.length; j++) {
            if (i == selectedIndices[j]) {
              idx = j;
              break;
            }
          }
          if (idx < 0) {
            indices[ofs++] = i;
          }
        }
        cgPalette.setSelectedIndices(indices);
        fireChangeListener();
      } else if (event.getSource() == bStoreSelect) {
        storeColorSelection();
      } else if (event.getSource() == bRestoreSelect) {
        if (!restoreColorSelection()) {
          showMessagePanel("Color selection is not available.");
        }
      } else if (event.getSource() == messageButton) {
        hideMessagePanel();
      }
    }

    // --------------------- End Interface ActionListener ---------------------

    // --------------------- Begin Interface ItemListener ---------------------

    @Override
    public void itemStateChanged(ItemEvent e) {
      if (e.getSource() == cbColorSort) {
        if (e.getStateChange() == ItemEvent.SELECTED) {
          final int[] selected = getSelectedIndices();
          updatePalette(getPalette());
          setSelectedIndices(selected);
        }
      }
    }

    // --------------------- End Interface ItemListener ---------------------

    private void init() {
      GridBagConstraints c = new GridBagConstraints();

      for (int i = 0; i < colorIndexLookup.length; i++) {
        colorIndexLookup[i] = i;
        colorIndexReverseLookup[i] = i;
      }

      // creating palette section
      cgPalette = new ColorGrid(256, null, Misc.getScaledDimension(ColorGrid.getDefaultColorEntryDimension()));
      cgPalette.setColorEntryHorizontalGap(4);
      cgPalette.setColorEntryVerticalGap(4);
      cgPalette.setSelectionMode(ColorGrid.SELECTION_MULTIPLE);
      cgPalette.setSelectionFrame(ColorGrid.Frame.SINGLE_LINE);
      cgPalette.addMouseOverListener(this);
      cgPalette.addActionListener(this);
      JPanel pPalette = new JPanel(new GridBagLayout());
      pPalette.setBorder(BorderFactory.createTitledBorder("Palette "));
      c = ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(0, 4, 2, 4), 0, 0);
      pPalette.add(cgPalette, c);

      // creating information panel
      JPanel pInfo = new JPanel(new GridBagLayout());
      pInfo.setBorder(BorderFactory.createTitledBorder("Information "));
      JLabel lInfoIndexTitle = new JLabel("Index:");
      JLabel lInfoRGBTitle = new JLabel("RGBA:");
      JLabel lInfoHexRGBTitle = new JLabel("Hex:");
      // XXX: making sure that the initial size of the components is big enough to hold all valid data
      lInfoIndex = new JLabel("1999");
      lInfoRGB = new JLabel(String.format(FMT_INFO_RGB, 1999, 1999, 1999, 1999));
      lInfoHexRGB = new JLabel(String.format(FMT_INFO_HEX_RGB, 0xAAA, 0xAAA, 0xAAA, 0xAAA));
      c = ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(0, 4, 0, 0), 0, 0);
      pInfo.add(lInfoIndexTitle, c);
      c = ViewerUtil.setGBC(c, 1, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(0, 8, 0, 4), 0, 0);
      pInfo.add(lInfoIndex, c);
      c = ViewerUtil.setGBC(c, 0, 1, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(4, 4, 0, 0), 0, 0);
      pInfo.add(lInfoRGBTitle, c);
      c = ViewerUtil.setGBC(c, 1, 1, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(4, 8, 0, 4), 0, 0);
      pInfo.add(lInfoRGB, c);
      c = ViewerUtil.setGBC(c, 0, 2, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(4, 4, 4, 0), 0, 0);
      pInfo.add(lInfoHexRGBTitle, c);
      c = ViewerUtil.setGBC(c, 1, 2, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(4, 8, 4, 4), 0, 0);
      pInfo.add(lInfoHexRGB, c);

      // creating button section
      JPanel pButtons = new JPanel(new GridBagLayout());
      pButtons.setBorder(BorderFactory.createTitledBorder("Select Colors "));
      bSelectAll = new JButton("Select all");
      bSelectAll.setMnemonic('a');
      bSelectAll.addActionListener(this);
      bSelectNone = new JButton("Select none");
      bSelectNone.setMnemonic('n');
      bSelectNone.addActionListener(this);
      bSelectInvert = new JButton("Invert selection");
      bSelectInvert.setMnemonic('i');
      bSelectInvert.addActionListener(this);
      bStoreSelect = new JButton("Store selection");
      bStoreSelect.addActionListener(this);
      bRestoreSelect = new JButton("Restore selection");
      bRestoreSelect.addActionListener(this);
      c = ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(4, 4, 0, 4), 0, 0);
      pButtons.add(bSelectAll, c);
      c = ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(4, 4, 0, 4), 0, 0);
      pButtons.add(bSelectNone, c);
      c = ViewerUtil.setGBC(c, 0, 2, 1, 1, 1.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(4, 4, 0, 4), 0, 0);
      pButtons.add(bSelectInvert, c);
      c = ViewerUtil.setGBC(c, 0, 3, 1, 1, 1.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(4, 4, 0, 4), 0, 0);
      pButtons.add(bStoreSelect, c);
      c = ViewerUtil.setGBC(c, 0, 4, 1, 1, 1.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(4, 4, 4, 4), 0, 0);
      pButtons.add(bRestoreSelect, c);

      // creating color sorting section
      final JPanel pColorSort = new JPanel(new GridBagLayout());
      pColorSort.setBorder(BorderFactory.createTitledBorder("Sort Colors "));
      final JLabel lColorSort = new JLabel("Sort by:");
      cbColorSort = new JComboBox<>(ColorSort.values());
      cbColorSort.setSelectedIndex(ColorSort.ORIGINAL.ordinal());
      cbColorSort.addItemListener(this);

      c = ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
          new Insets(0, 4, 0, 0), 0, 0);
      pColorSort.add(lColorSort, c);
      c = ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(4, 4, 4, 4), 0, 0);
      pColorSort.add(cbColorSort, c);

      // putting sidebar together
      JPanel pSideBar = new JPanel(new GridBagLayout());
      c = ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(0, 0, 0, 0), 0, 0);
      pSideBar.add(pInfo, c);
      c = ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(8, 0, 0, 0), 0, 0);
      pSideBar.add(pButtons, c);
      c = ViewerUtil.setGBC(c, 0, 2, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(8, 0, 0, 0), 0, 0);
      pSideBar.add(pColorSort, c);

      // putting all together
      JPanel pMain = new JPanel(new GridBagLayout());
      c = ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.NONE,
          new Insets(0, 0, 0, 0), 0, 0);
      pMain.add(pPalette, c);
      c = ViewerUtil.setGBC(c, 1, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(0, 4, 0, 0), 0, 0);
      pMain.add(pSideBar, c);

      // and adding to main panel
      setBorder(BorderFactory.createEmptyBorder(4, 4, 4, 4));
      add(pMain, CARD_MAIN);

      // message box panel
      JPanel pMessage = new JPanel(new GridBagLayout());
      messageLabel = new JLabel("", JLabel.CENTER);
      messageButton = new JButton("OK");
      final Insets margin = messageButton.getMargin();
      margin.left += 8;
      margin.right += 8;
      messageButton.setMargin(margin);
      messageButton.addActionListener(this);
      c = ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.CENTER, GridBagConstraints.NONE,
          new Insets(8, 8, 8, 8), 0, 0);
      pMessage.add(messageLabel, c);
      c = ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.CENTER, GridBagConstraints.NONE,
          new Insets(8, 8, 8, 8), 0, 0);
      pMessage.add(messageButton, c);
      add(pMessage, CARD_MESSAGE);

      hideMessagePanel();
      updateInfoBox(-1);
    }

    // Updates the information panel
    private void updateInfoBox(int index) {
      if (index >= 0 && index < cgPalette.getColorCount()) {
        Color c = cgPalette.getColor(index);
        final int srcIdx = (index >= 0 && index < colorIndexReverseLookup.length) ? colorIndexReverseLookup[index] : -1;
        setLabelText(lInfoIndex, Integer.toString(srcIdx), null);
        setLabelText(lInfoRGB, String.format(FMT_INFO_RGB, c.getRed(), c.getGreen(), c.getBlue(), c.getAlpha()), null);
        setLabelText(lInfoHexRGB, String.format(FMT_INFO_HEX_RGB, c.getRed(), c.getGreen(), c.getBlue(), c.getAlpha()),
            null);
      } else {
        setLabelText(lInfoIndex, "", null);
        setLabelText(lInfoRGB, "", null);
        setLabelText(lInfoHexRGB, "", null);
      }
    }

    // Sets a new text to the specified JLabel component while retaining its preferred size
    private void setLabelText(JLabel c, String text, Dimension d) {
      if (c != null) {
        if (text == null) {
          text = "";
        }
        if (d == null) {
          d = c.getPreferredSize();
        }
        c.setText(text);
        c.setPreferredSize(d);
      }
    }

    private void fireChangeListener() {
      ChangeEvent event = new ChangeEvent(this);
      for (ChangeListener listChangeListener : listChangeListeners) {
        listChangeListener.stateChanged(event);
      }
    }

    /** Returns whether a color selection is currently stored. */
    private boolean isColorSelectionStored() {
      return colorSelection != null;
    }

    /** Stores the current color selection. A previous selection will be overwritten. */
    private void storeColorSelection() {
      colorSelection = getSelectedIndices();
    }

    /** Restores a global color selection. Returns {@code true} if a selection could be restored. */
    private boolean restoreColorSelection() {
      if (isColorSelectionStored()) {
        setSelectedIndices(colorSelection);
        return true;
      }
      return false;
    }

    /** Displays a message panel with button. */
    private void showMessagePanel(String message) {
      final String labelMessage = (message != null) ? message : "(null)";
      SwingUtilities.invokeLater(() -> {
        messageLabel.setText(labelMessage);
        messageButton.setEnabled(true);
        ((CardLayout)getLayout()).show(this, CARD_MESSAGE);
        getRootPane().setDefaultButton(messageButton);
        messageButton.requestFocusInWindow();
      });
    }

    /** Hides message panel if it is visible. */
    private void hideMessagePanel() {
      SwingUtilities.invokeLater(() -> {
        getRootPane().setDefaultButton(null);
        messageButton.setEnabled(false);
        ((CardLayout)getLayout()).show(this, CARD_MAIN);
      });
    }

    /**
     * Sorts the specified palette by the given sorting method.
     *
     * @param palette Original palette to sort.
     * @param lookup  A lookup array: original color -> sorted color
     * @param rlookup A reversed lookup array: sorted color -> original color
     * @return Sorted palette.
     */
    private static Color[] getSortedPalette(ColorSort sort, int[] palette, int[] lookup, int[] rlookup) {
      if (sort == null) {
        sort = ColorSort.ORIGINAL;
      }
      if (palette == null) {
        palette = new int[0];
      }

      // creating sorted list
      final Map<Color, Integer> cache = new IdentityHashMap<>(344);
      final List<Color> dstColors = new ArrayList<>(256);
      for (int i = 1; i < 256; i++) {
        final int argb = (i < palette.length) ? palette[i] : Color.BLACK.getRGB();
        final Color color = new Color(argb, true);
        cache.put(color, i);
        dstColors.add(color);
      }
      dstColors.sort(sort.getComparator());

      // palette index 0 is fixed
      final int argb = (palette.length > 0) ? palette[0] : Color.GREEN.getRGB();
      final Color color = new Color(argb, true);
      dstColors.add(0, color);
      if (lookup != null && lookup.length > 0) {
        lookup[0] = 0;
      }
      if (rlookup != null && rlookup.length > 0) {
        rlookup[0] = 0;
      }

      // initializing lookup tables
      if (lookup != null || rlookup != null) {
        for (int dstIdx = 1; dstIdx < 256; dstIdx++) {
          final Color c = dstColors.get(dstIdx);
          final int srcIdx = cache.getOrDefault(c, dstIdx);
          if (lookup != null && srcIdx < lookup.length) {
            lookup[srcIdx] = dstIdx;
          }
          if (rlookup != null && dstIdx < rlookup.length) {
            rlookup[dstIdx] = srcIdx;
          }
        }
      }

      return dstColors.toArray(new Color[dstColors.size()]);
    }
  }
}

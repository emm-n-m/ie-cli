// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bam;

import java.awt.BorderLayout;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.awt.event.KeyEvent;
import java.awt.image.DataBuffer;
import java.awt.image.IndexColorModel;
import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

import javax.swing.AbstractAction;
import javax.swing.JButton;
import javax.swing.JCheckBox;
import javax.swing.JComponent;
import javax.swing.JDialog;
import javax.swing.JLabel;
import javax.swing.JOptionPane;
import javax.swing.JPanel;
import javax.swing.KeyStroke;
import javax.swing.WindowConstants;
import javax.swing.filechooser.FileNameExtensionFilter;

import org.infinity.gui.ViewerUtil;
import org.infinity.gui.WindowBlocker;
import org.infinity.resource.Profile;
import org.infinity.resource.ResourceFactory;
import org.infinity.resource.graphics.BamDecoder;
import org.infinity.resource.graphics.BamV1Decoder;
import org.infinity.resource.graphics.PseudoBamDecoder;
import org.infinity.resource.graphics.PseudoBamDecoder.PseudoBamCycleEntry;
import org.infinity.resource.graphics.PseudoBamDecoder.PseudoBamFrameEntry;
import org.infinity.resource.key.FileResourceEntry;
import org.infinity.resource.key.ResourceEntry;
import org.infinity.util.IniMap;
import org.infinity.util.IniMapEntry;
import org.infinity.util.IniMapSection;
import org.infinity.util.Logger;
import org.infinity.util.Misc;
import org.infinity.util.io.FileEx;
import org.infinity.util.io.FileManager;
import org.infinity.util.io.StreamUtils;

/**
 * Provides methods for importing or exporting BAM configuration data via INI file, such as frame sources, center
 * position data or cycle definitions.
 */
class Exporter extends JDialog implements ActionListener {
  /*
   * @formatter:off
   * INI format:
   * 1. Section "[Global]" (mandatory)
   *  - contains a single entry "version" with a version number
   * 2. Section "[Frames]" (optional)
   *  - contains any number of frame source definitions in the format
   *    - key: zero-based frame index
   *    - value: path to graphics file (absolute or relative to ini file),
   *             optionally separated by colon ':' followed by a frame index
   *             (only for input files containing multiple frames, default: 0)
   *      Example: 0=c:/myfolder/myfile.bam:12 <- to load frame 12 of myfile.bam
   * 3. Section "[Center]" (optional)
   *  - contains any number of center position entries for individual frames in the format
   *    - key: zero-based frame index
   *    - value: a sequence of two numbers for x and y, separated by comma ','
   *      Example: 0=12,-55 <- for position [12.-55]
   * 4. Section "[Cycles]" (optional)
   *  - contains cycle definitions for the BAM in the format
   *    - key: zero-based cycle index
   *    - value: a sequence of numbers specifying frame indices, separated by comma ','
   *      Example: 0=0,1,2,3,4,5,90,91,92,93
   * 5. Section "[Filters]" (optional)
   *  - contains a list of filters to apply, including filter configurations
   *  - uses name and config entries
   *  - name key: name_n (where n is a positive number)
   *  - name value: the filter name
   *  - config key: config_n (where n is a positive number)
   *  - config value: a configuration string (can be empty)
   *  - enabled key: enabled_n (where n is a positive number)
   *  - enabled value: a single integer number that indicates whether the filter is enabled (0: false, !0: true)
   *  - Example:
   *      name_0=Brightness/Contrast/Gamma
   *      config_0=25;100;128;[0,18,19,20,192,193,194,195]
   *      enabled_0=1
   * @formatter:on
   */
  private static final String SECTION_GLOBAL      = "Global"; // global section name
  private static final String SECTION_FRAMES      = "Frames"; // frames section name
  private static final String SECTION_CENTER      = "Center"; // center point section name
  private static final String SECTION_CYCLES      = "Cycles"; // cycles section name
  private static final String SECTION_FILTERS     = "Filters"; // filters section name
  private static final String KEY_VERSION         = "version"; // key in global section
  private static final String KEY_FILTER_NAME     = "name_"; // key in filters section
  private static final String KEY_FILTER_CONFIG   = "config_"; // key in filters section
  private static final String KEY_FILTER_ENABLED  = "enabled_"; // key in filters sections
  private static final char SEPARATOR_FRAME       = ':'; // used in frame source definition to separate frame name from index
  private static final char SEPARATOR_NUMBER      = ','; // number separator for cycle definitions or center point data
  private static final int VERSION                = 2; // max. supported file version
  private static final String QUESTION_EXPORT     = "What do you want to export?";
  private static final String QUESTION_IMPORT     = "What do you want to import?";

  private final JLabel lSelect = new JLabel();
  private final JCheckBox cbFrames = new JCheckBox("Frame source files", true);
  private final JCheckBox cbCenter = new JCheckBox("Frame center coordinates", true);
  private final JCheckBox cbCycles = new JCheckBox("Cycle definitions", true);
  private final JCheckBox cbFilters = new JCheckBox("Filter configurations", true);
  private final JButton bAccept = new JButton("Accept");
  private final JButton bCancel = new JButton("Cancel");
  private final ConvertToBam bam;

  private IniMapSection sectionFrames;
  private IniMapSection sectionCenter;
  private IniMapSection sectionCycles;
  private IniMapSection sectionFilters;
  private boolean accepted;
  private int version;

  /** Returns a extension filter for INI files. */
  private static FileNameExtensionFilter getIniFilter() {
    return new FileNameExtensionFilter("INI files (*.ini)", "ini");
  }

  public Exporter(ConvertToBam bam) {
    super(bam, true);
    this.bam = bam;
    init();
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent event) {
    if (event.getSource() == bAccept) {
      acceptDialog();
    } else if (event.getSource() == bCancel) {
      cancel();
    } else if (event.getSource() instanceof JCheckBox) {
      bAccept.setEnabled(
          cbFrames.isSelected() || cbCenter.isSelected() || cbCycles.isSelected() || cbFilters.isSelected());
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  /** Must be called at the end to clean up dialog resources. */
  public void close() {
    dispose();
  }

  /**
   * Opens dialog to choose what to export and exports selected data. Returns whether export was successful.
   */
  public boolean exportData(boolean silent) {
    resetData();

    // trying to determine default output filename
    Path root = getDefaultIniName("data.ini");
    Path outFile = ConvertToBam.getSaveFileName(bam, "Export BAM session", root, new FileNameExtensionFilter[] { getIniFilter() },
        0);
    if (outFile != null) {
      outFile = StreamUtils.replaceFileExtension(outFile, "ini");
      bam.updateRecentSession(outFile);
      if (getSelection(true)) {
        try {
          WindowBlocker.blockWindow(bam, true);
          return saveData(outFile, silent);
        } finally {
          WindowBlocker.blockWindow(bam, false);
        }
      }
    }
    return false;
  }

  /**
   * Opens dialog to choose what to import and imports selected data. Returns whether import was successful.
   */
  public boolean importData(boolean silent) {
    resetData();

    Path[] files = ConvertToBam.getOpenFileName(bam, "Import BAM session", null, false,
        new FileNameExtensionFilter[] { getIniFilter() }, 0);
    if (files != null && files.length > 0) {
      if (!FileEx.create(files[0]).isFile()) {
        files[0] = StreamUtils.replaceFileExtension(files[0], "ini");
      }
      if (loadData(files[0], silent)) {
        bam.updateRecentSession(files[0]);
        if (getSelection(false)) {
          try {
            WindowBlocker.blockWindow(bam, true);
            return applyData(silent);
          } finally {
            WindowBlocker.blockWindow(bam, false);
          }
        }
      }
    }
    return false;
  }

  /**
   * Imports selected data from the specified session file. Returns whether import was successful.
   */
  public boolean importData(Path session, boolean silent) {
    if (session != null) {
      resetData();
      if (loadData(session, silent)) {
        bam.updateRecentSession(session);
        if (getSelection(false)) {
          try {
            WindowBlocker.blockWindow(bam, true);
            return applyData(silent);
          } finally {
            WindowBlocker.blockWindow(bam, false);
          }
        }
      }
    }
    return false;
  }

  /** Loads data from the specified file without user-interaction and optionally without feedback. */
  private boolean loadData(Path inFile, boolean silent) {
    if (inFile != null) {
      IniMap ini = new IniMap(new FileResourceEntry(inFile), true);

      try {
        // checking integrity
        if (ini.getSection(SECTION_GLOBAL) == null || ini.getSection(SECTION_GLOBAL).getEntry(KEY_VERSION) == null) {
          throw new Exception("Invalid BAM session file.");
        }

        final int version = Misc.toNumber(ini.getSection(SECTION_GLOBAL).getEntry(KEY_VERSION).getValue(), -1);
        if (version < 0 || version > VERSION) {
          throw new Exception("Invalid or unsupported file version.");
        }
        this.version = version;

        if (ini.getSection(SECTION_FRAMES) != null) {
          if (!loadFrameData(ini.getSection(SECTION_FRAMES), inFile)) {
            throw new Exception("Error loading frame source files.");
          }
        }

        if (ini.getSection(SECTION_CENTER) != null) {
          if (!loadCenterData(ini.getSection(SECTION_CENTER))) {
            throw new Exception("Error loading frame center coordinates.");
          }
        }

        if (ini.getSection(SECTION_CYCLES) != null) {
          if (!loadCycleData(ini.getSection(SECTION_CYCLES))) {
            throw new Exception("Error loading cycle definitions.");
          }
        }

        if (ini.getSection(SECTION_FILTERS) != null) {
          if (!loadFilterData(ini.getSection(SECTION_FILTERS))) {
            throw new Exception("Error loading filters.");
          }
        }

        return true;
      } catch (Exception e) {
        // parsing failed
        resetData();
        if (!silent && e.getMessage() != null && !e.getMessage().isEmpty()) {
          JOptionPane.showMessageDialog(bam, e.getMessage(), "Error", JOptionPane.ERROR_MESSAGE);
        }
      }
    }
    return false;
  }

  private boolean loadFrameData(IniMapSection frames, Path iniFile) throws Exception {
    if (frames != null && frames.getName().equalsIgnoreCase(SECTION_FRAMES)) {
      final List<IniMapEntry> frameEntries = new ArrayList<>();
      final Path basePath;
      if (iniFile != null && iniFile.getParent() != null) {
          basePath = iniFile.getParent();
      } else {
        basePath = Profile.getGameRoot();
      }

      for (final IniMapEntry entry : frames) {
        if (Misc.toNumber(entry.getKey(), -1) < 0) {
          throw new Exception("Invalid key value found at line " + (entry.getLine() + 1));
        }
        String value = entry.getValue().trim();
        if (value.isEmpty()) {
          throw new Exception("Empty frame source path found at line " + (entry.getLine() + 1));
        }

        int sepIdx = value.lastIndexOf(SEPARATOR_FRAME);
        int frameIdx = 0;
        if (sepIdx >= 0) {
          frameIdx = Misc.toNumber(value.substring(sepIdx + 1), -1);
          value = value.substring(0, sepIdx);
        }
        if (frameIdx < 0) {
          throw new Exception("Invalid frame index at line " + (entry.getLine() + 1));
        }

        final String newValue;
        if (value.startsWith(ConvertToBam.BAM_FRAME_PATH_BIFF)) {
          // game resource
          String resName = value.substring(ConvertToBam.BAM_FRAME_PATH_BIFF.length());
          if (!ResourceFactory.resourceExists(resName)) {
            throw new Exception("Frame source path not found at line " + (entry.getLine() + 1));
          }
          newValue = value + ':' + frameIdx;
        } else {
          // external file path
          Path file = Paths.get(value);
          if (!file.isAbsolute()) {
            file = basePath.resolve(file);
          }
          file = file.normalize();
          if (!FileEx.create(file).isFile()) {
            throw new Exception("Frame source path not found at line " + (entry.getLine() + 1));
          }
          newValue = file.toString() + ':' + frameIdx;
        }
        frameEntries.add(new IniMapEntry(entry.getKey(), newValue, entry.getLine()));
      }
      sectionFrames = new IniMapSection(frames.getName(), frames.getLine(), frameEntries);
      return true;
    }
    return true;
  }

  private boolean loadCenterData(IniMapSection centers) throws Exception {
    if (centers != null && centers.getName().equalsIgnoreCase(SECTION_CENTER)) {
      for (final IniMapEntry entry : centers) {
        if (Misc.toNumber(entry.getKey(), -1) < 0) {
          throw new Exception("Invalid key value found at line " + (entry.getLine() + 1));
        }
        if (!entry.getValue().trim().matches("-?\\d+\\s*,\\s*-?\\d+")) {
          throw new Exception("Invalid value found at line " + (entry.getLine() + 1));
        }
      }
      sectionCenter = centers;
      return true;
    }
    return false;
  }

  private boolean loadCycleData(IniMapSection cycles) throws Exception {
    if (cycles != null && cycles.getName().equalsIgnoreCase(SECTION_CYCLES)) {
      for (final IniMapEntry entry : cycles) {
        if (Misc.toNumber(entry.getKey(), -1) < 0) {
          throw new Exception("Invalid key value found at line " + (entry.getLine() + 1));
        }
        if (!entry.getValue().trim().matches("(\\d+\\s*(,\\s*\\d+\\s*)*)?")) {
          throw new Exception("Invalid value found at line " + (entry.getLine() + 1));
        }
      }
      sectionCycles = cycles;
      return true;
    }
    return false;
  }

  private boolean loadFilterData(IniMapSection filters) throws Exception {
    final ArrayList<String> keyMatches = new ArrayList<>(4);
    keyMatches.add(KEY_FILTER_CONFIG + "\\d+");
    if (getVersion() >= 2) {
      keyMatches.add(KEY_FILTER_ENABLED + "\\d+");
    }

    if (filters != null && filters.getName().equalsIgnoreCase(SECTION_FILTERS)) {
      for (final IniMapEntry entry : filters) {
        String key = entry.getKey().trim();
        String value = entry.getValue().trim();
        if (key.matches(KEY_FILTER_NAME + "\\d+")) {
          if (BamFilterFactory.getFilterInfo(value) == null) {
            throw new Exception(
                "BAM filter \"" + value.substring(0, Math.min(value.length(), 256)) + "\" does not exist.");
          }
        } else {
          boolean match = false;
          for (final String regex : keyMatches) {
            if (key.matches(regex)) {
              match = true;
              break;
            }
          }
          if (!match) {
            throw new Exception("Invalid key value found at line " + (entry.getLine() + 1));
          }
        }
      }
      sectionFilters = filters;
      return true;
    }
    return false;
  }

  /** Applies available data to the converter without user-interaction and optionally without feedback. */
  private boolean applyData(boolean silent) {
    bam.previewStop();
    bam.outputSetModified(true);

    try {
      if (sectionFrames != null) {
        if (!applyFramesData(silent)) {
          throw new Exception("Error adding frame entries.");
        }
      }
      if (sectionCenter != null) {
        if (!applyCenterData(silent)) {
          throw new Exception("Error applying frame center coordinates.");
        }
      }
      if (sectionCycles != null) {
        if (!applyCycleData(silent)) {
          throw new Exception("Error adding cycle definitions.");
        }
      }
      if (sectionFilters != null) {
        if (!applyFilterData(silent)) {
          throw new Exception("Error adding filters.");
        }
      }
      return true;
    } catch (Exception e) {
      resetData();
      if (!silent && e.getMessage() != null && !e.getMessage().isEmpty()) {
        WindowBlocker.blockWindow(bam, false);
        JOptionPane.showMessageDialog(bam, e.getMessage(), "Error", JOptionPane.ERROR_MESSAGE);
      }
    }
    return false;
  }

  private boolean applyFramesData(boolean silent) throws Exception {
    /* Storage for ResourceEntry and frame index for convenience. */
    class SourceFrame {
      public final ResourceEntry entry;
      public final int index;

      public SourceFrame(ResourceEntry entry, int index) {
        this.entry = entry;
        this.index = index;
      }
    }

    /* Primarily used for caching BAM decoder instances. */
    class SourceData {
      public final boolean isBam;
      // bam-specific
      public final BamDecoder decoder;
      public final BamDecoder.BamControl control;
      public final IndexColorModel cm;
      // image-specific
      public final Path file;

      public SourceData(BamDecoder decoder) {
        this.isBam = true;
        this.decoder = decoder;
        this.control = this.decoder.createControl();
        if (this.decoder instanceof BamV1Decoder) {
          int[] palette = ((BamV1Decoder.BamV1Control) control).getPalette();
          int transColor = ((BamV1Decoder.BamV1Control) control).getTransparencyIndex();
          this.cm = new IndexColorModel(8, 256, palette, 0, ConvertToBam.getUseAlpha(), transColor, DataBuffer.TYPE_BYTE);
        } else {
          this.cm = null;
        }
        this.file = null;
      }

      public SourceData(Path image) {
        this.isBam = false;
        this.decoder = null;
        this.control = null;
        this.cm = null;
        this.file = image;
      }
    }

    if (sectionFrames != null) {
      // preparing frames
      int entryCount = sectionFrames.getEntryCount();
      SourceFrame[] frames = new SourceFrame[entryCount];
      for (final IniMapEntry entry : sectionFrames) {
        // checking list indices
        int listIndex = Misc.toNumber(entry.getKey(), -1);
        if (listIndex < 0 || listIndex >= entryCount) {
          throw new Exception(
              "Target frame index out of range [: " + listIndex + "] at line " + (entry.getLine() + 1));
        }

        // checking frame source paths and indices
        String value = entry.getValue().trim();
        int frameIndex = -1;
        int sepIdx = value.lastIndexOf(SEPARATOR_FRAME);
        if (sepIdx >= 0) {
          frameIndex = Misc.toNumber(value.substring(sepIdx + 1), -1);
          value = value.substring(0, sepIdx);
        }
        if (frameIndex < 0 || value.isEmpty()) {
          throw new Exception(
              "Source frame index out of range [: " + listIndex + "] at line " + (entry.getLine() + 1));
        }

        ResourceEntry resource = null;
        if (value.startsWith(ConvertToBam.BAM_FRAME_PATH_BIFF)) {
          value = value.substring(ConvertToBam.BAM_FRAME_PATH_BIFF.length());
          if (ResourceFactory.resourceExists(value)) {
            resource = ResourceFactory.getResourceEntry(value);
          }
        } else {
          Path file = FileManager.resolve(value);
          if (FileEx.create(file).isFile()) {
            resource = new FileResourceEntry(file);
          }
        }
        if (resource == null) {
          throw new Exception("Resource does not exist at line " + (entry.getLine() + 1));
        }

        frames[listIndex] = new SourceFrame(resource, frameIndex);
      }
      for (int i = 0; i < frames.length; i++) {
        if (frames[i] == null) {
          throw new Exception("Undefined target frame index " + i);
        }
      }

      bam.filterRemoveAll();
      bam.cyclesRemoveAll();
      bam.framesRemoveAll();
      bam.getPaletteDialog().clear();

      // applying frames
      HashMap<ResourceEntry, SourceData> sourceMap = new HashMap<>();
      for (int i = 0; i < frames.length; i++) {
        SourceFrame frame = frames[i];
        SourceData data = sourceMap.get(frame.entry);
        if (data == null) {
          if (BamDecoder.isValid(frame.entry)) {
            data = new SourceData(BamDecoder.loadBam(frame.entry));
          } else {
            data = new SourceData(frame.entry.getActualPath());
          }
          sourceMap.put(frame.entry, data);
        }
        if (data.isBam) {
          bam.framesAddBamFrame(i, data.decoder, data.control, frame.index, data.cm);
        } else {
          bam.framesAddImage(i, data.file, frame.index);
        }
      }
      bam.updateFramesList();
      return true;
    }
    return false;
  }

  private boolean applyCenterData(boolean silent) throws Exception {
    if (sectionCenter != null) {
      for (final IniMapEntry entry : sectionCenter) {
        int listIndex = Misc.toNumber(entry.getKey(), -1);
        if (listIndex >= 0 && listIndex < bam.getFramesModel().getSize()) {
          String[] numbers = entry.getValue().trim().split(Character.toString(SEPARATOR_NUMBER));
          if (numbers.length >= 2) {
            int x = Misc.toNumber(numbers[0].trim(), Integer.MIN_VALUE);
            int y = Misc.toNumber(numbers[1].trim(), Integer.MIN_VALUE);
            if (x != Integer.MIN_VALUE && y != Integer.MIN_VALUE) {
              PseudoBamFrameEntry bfe = Objects.requireNonNull(bam.getFramesModel().getElementAt(listIndex));
              bfe.setCenterX(x);
              bfe.setCenterY(y);
            }
          }
        }
      }
      bam.updateFramesList();
      return true;
    }
    return false;
  }

  private boolean applyCycleData(boolean silent) throws Exception {
    if (sectionCycles != null) {
      if (bam.getFramesModel().getSize() == 0) {
        throw new Exception("Unable to add cycle definitions. No frames available.");
      }

      // preparing cycle definitions
      final HashMap<Integer, int[]> cycles = new HashMap<>();
      int maxCycle = -1;
      for (final IniMapEntry entry : sectionCycles) {
        int cycleIndex = Misc.toNumber(entry.getKey(), -1);
        if (cycleIndex >= 0) {
          String value = entry.getValue().trim();
          String[] values = (value.isEmpty()) ? new String[0] : value.split(Character.toString(SEPARATOR_NUMBER));
          int[] cycleList = new int[values.length];
          for (int j = 0; j < cycleList.length; j++) {
            int n = Misc.toNumber(values[j].trim(), -1);
            n = Math.max(0, Math.min(bam.getFramesModel().getSize() - 1, n));
            cycleList[j] = n;
          }
          cycles.put(cycleIndex, cycleList);
          maxCycle = Math.max(maxCycle, cycleIndex);
        }
      }
      if (maxCycle < 0) {
        // no cycles defined -> return successfully
        return true;
      }

      // post-processing
      int[][] cycleArray = new int[maxCycle + 1][];
      for (Map.Entry<Integer, int[]> entry : cycles.entrySet()) {
        cycleArray[entry.getKey()] = entry.getValue();
      }

      bam.filterRemoveAll();
      bam.cyclesRemoveAll();

      // applying cycle definitions
      final int[] emptyCycle = new int[0];
      for (int[] cycle : cycleArray) {
        int[] curCycle = (cycle != null) ? cycle : emptyCycle;
        bam.getCyclesModel().add(curCycle);
      }

      bam.updateCyclesList();
      return true;
    }
    return false;
  }

  private boolean applyFilterData(boolean silent) throws Exception {
    class Config {
      public String name;
      public String param;
      public boolean enabled;

      public Config() {
        enabled = true;
      }
    }

    if (sectionFilters != null) {
      if (bam.getFramesModel().getSize() == 0) {
        throw new Exception("Unable to add filters. No frames available.");
      }

      // preparing filter list
      final HashMap<Integer, Config> filterMap = new HashMap<>();
      int maxIndex = -1;
      for (final IniMapEntry entry : sectionFilters) {
        String key = entry.getKey();
        if (key.startsWith(KEY_FILTER_NAME)) {
          final int idx = Misc.toNumber(key.substring(KEY_FILTER_NAME.length()), -1);
          if (idx >= 0) {
            String name = entry.getValue().trim();
            Config config = filterMap.get(idx);
            if (config == null) {
              config = new Config();
              filterMap.put(idx, config);
            }
            config.name = name;
            maxIndex = Math.max(maxIndex, idx);
          }
        } else if (key.startsWith(KEY_FILTER_CONFIG)) {
          final int idx = Misc.toNumber(key.substring(KEY_FILTER_CONFIG.length()), -1);
          if (idx >= 0) {
            String param = entry.getValue().trim();
            Config config = filterMap.get(idx);
            if (config == null) {
              config = new Config();
              filterMap.put(idx, config);
            }
            config.param = param;
            maxIndex = Math.max(maxIndex, idx);
          }
        } else if (getVersion() >= 2 && key.startsWith(KEY_FILTER_ENABLED)) {
          final int idx = Misc.toNumber(key.substring(KEY_FILTER_ENABLED.length()), -1);
          if (idx >= 0) {
            final boolean enabled = (Misc.toNumber(entry.getValue(), 1) != 0);
            Config config = filterMap.get(idx);
            if (config == null) {
              config = new Config();
              filterMap.put(idx, config);
            }
            config.enabled = enabled;
            maxIndex = Math.max(maxIndex, idx);
          }
        }
      }
      if (maxIndex < 0) {
        // no filters defined -> return successfully
        return true;
      }

      // post-processing data
      Config[] configArray = new Config[maxIndex + 1];
      for (Map.Entry<Integer, Config> entry : filterMap.entrySet()) {
        final Config config = entry.getValue();
        if (config.name != null) {
          if (config.param == null) {
            config.param = "";
          }
          configArray[entry.getKey()] = config;
        }
      }

      // applying filter list
      bam.filterRemoveAll();
      for (Config config : configArray) {
        if (config != null) {
          BamFilterFactory.FilterInfo info = BamFilterFactory.getFilterInfo(config.name);
          if (info != null) {
            BamFilterBase filter = bam.filterAdd(info);
            if (filter != null) {
              filter.setConfiguration(config.param);
              filter.setEnabled(config.enabled);
            }
          }
        }
      }

      bam.updateFilterList();
      return true;
    }
    return false;
  }

  /** Saves data to specified INI file without user-interaction and optionally without feedback. */
  private boolean saveData(Path outFile, boolean silent) {
    boolean retVal = false;
    if (outFile != null) {
      final StringBuilder sb = new StringBuilder();

      // creating global section
      sb.append('[').append(SECTION_GLOBAL).append(']').append(Misc.LINE_SEPARATOR);
      sb.append(KEY_VERSION).append('=').append(VERSION).append(Misc.LINE_SEPARATOR);
      sb.append(Misc.LINE_SEPARATOR);
      this.version = VERSION;

      // creating frames section
      if (isFramesSelected()) {
        final Path basePath = outFile.getParent() != null ? outFile.getParent() : null;
        sb.append('[').append(SECTION_FRAMES).append(']').append(Misc.LINE_SEPARATOR);
        for (int i = 0; i < bam.getFramesModel().getSize(); i++) {
          PseudoBamFrameEntry entry = bam.getFramesModel().getElementAt(i);
          String path = entry.getOption(ConvertToBam.BAM_FRAME_OPTION_PATH).toString();
          if (!path.startsWith(ConvertToBam.BAM_FRAME_PATH_BIFF)) {
            // regular file path: relativize if possible
            Path framePath = Paths.get(path);
            if (framePath.startsWith(basePath)) {
              framePath = basePath.relativize(framePath);
            }
            path = framePath.toString();
          }
          int index = ((Number) entry.getOption(ConvertToBam.BAM_FRAME_OPTION_SOURCE_INDEX)).intValue();
          sb.append(i).append('=').append(path).append(SEPARATOR_FRAME).append(index);
          sb.append(Misc.LINE_SEPARATOR);
        }
        sb.append(Misc.LINE_SEPARATOR);
      }

      // creating center section
      if (isCenterSelected()) {
        sb.append('[').append(SECTION_CENTER).append(']').append(Misc.LINE_SEPARATOR);
        for (int i = 0; i < bam.getFramesModel().getSize(); i++) {
          PseudoBamFrameEntry entry = bam.getFramesModel().getElementAt(i);
          sb.append(i).append('=');
          sb.append(entry.getCenterX()).append(SEPARATOR_NUMBER).append(entry.getCenterY());
          sb.append(Misc.LINE_SEPARATOR);
        }
        sb.append(Misc.LINE_SEPARATOR);
      }

      // creating cycles section
      if (isCyclesSelected()) {
        sb.append('[').append(SECTION_CYCLES).append(']').append(Misc.LINE_SEPARATOR);
        for (int i = 0; i < bam.getCyclesModel().getSize(); i++) {
          PseudoBamCycleEntry entry = bam.getCyclesModel().getElementAt(i);
          sb.append(i).append('=');
          for (int j = 0; j < entry.size(); j++) {
            if (j > 0) {
              sb.append(SEPARATOR_NUMBER);
            }
            sb.append(entry.get(j));
          }
          sb.append(Misc.LINE_SEPARATOR);
        }
        sb.append(Misc.LINE_SEPARATOR);
      }

      // creating filters section
      if (isFiltersSelected()) {
        sb.append('[').append(SECTION_FILTERS).append(']').append(Misc.LINE_SEPARATOR);
        for (int i = 0; i < bam.getFilterModel().getSize(); i++) {
          BamFilterBase filter = bam.getFilterModel().getElementAt(i);
          sb.append(KEY_FILTER_NAME).append(i).append('=').append(filter.getName()).append(Misc.LINE_SEPARATOR);
          sb.append(KEY_FILTER_CONFIG).append(i).append('=').append(filter.getConfiguration())
              .append(Misc.LINE_SEPARATOR);
          sb.append(KEY_FILTER_ENABLED).append(i).append('=').append(filter.isEnabled() ? 1 : 0)
              .append(Misc.LINE_SEPARATOR);
        }
        sb.append(Misc.LINE_SEPARATOR);
      }

      // writing data to disk
      try (BufferedWriter bw = Files.newBufferedWriter(outFile)) {
        bw.write(sb.toString());
        if (!silent) {
          JOptionPane.showMessageDialog(bam, "Export completed.", "Message", JOptionPane.INFORMATION_MESSAGE);
        }
        retVal = true;
      } catch (IOException e) {
        Logger.error(e);
        if (!silent) {
          JOptionPane.showMessageDialog(bam, "Error exporting BAM session.", "Error", JOptionPane.ERROR_MESSAGE);
        }
      }
    }
    return retVal;
  }

  /** Clears all BAM session data. */
  private void resetData() {
    sectionFrames = null;
    sectionCenter = null;
    sectionCycles = null;
    sectionFilters = null;
  }

  /** Shows options dialog and returns whether user selected "Accept" or "Cancel". */
  private boolean getSelection(boolean isExport) {
    if (isExport) {
      setTitle("Export BAM session");
      lSelect.setText(QUESTION_EXPORT);
      cbFrames.setEnabled(bam.getFramesModel().getSize() > 0);
      cbCenter.setEnabled(bam.getFramesModel().getSize() > 0);
      cbCycles.setEnabled(bam.getCyclesModel().getSize() > 0);
      cbFilters.setEnabled(bam.getFilterModel().getSize() > 0);
    } else {
      setTitle("Import BAM session");
      lSelect.setText(QUESTION_IMPORT);
      cbFrames.setEnabled(sectionFrames != null);
      cbCenter.setEnabled(sectionCenter != null);
      cbCycles.setEnabled(sectionCycles != null);
      cbFilters.setEnabled(sectionFilters != null);
    }
    cbFrames.setSelected(cbFrames.isEnabled());
    cbCenter.setSelected(cbCenter.isEnabled());
    cbCycles.setSelected(cbCycles.isEnabled());
    cbFilters.setSelected(cbFilters.isEnabled());
    pack();
    setLocationRelativeTo(bam);
    bAccept.requestFocusInWindow();
    setVisible(true);

    return isAccepted();
  }

  /** Attempts to determine a fitting default name for the ini file. */
  private Path getDefaultIniName(String defaultName) {
    Path retVal = null;
    if (bam.getFramesModel().getSize() > 0) {
      String name = bam.getFramesModel().getElementAt(0).getOption(PseudoBamDecoder.OPTION_STRING_LABEL).toString();
      if (name != null) {
        if (name.indexOf(':') > 0) {
          name = name.substring(0, name.indexOf(':'));
        }
        if (!name.isEmpty()) {
          Path file = ConvertToBam.getCurrentPath().resolve(name);
          retVal = StreamUtils.replaceFileExtension(file, "ini");
        }
      }
    }
    if (retVal == null) {
      Path file = ConvertToBam.getCurrentPath().resolve(defaultName);
      retVal = StreamUtils.replaceFileExtension(file, "ini");
    }

    return retVal;
  }

  /** Returns whether the dialog options have been accepted. */
  private boolean isAccepted() {
    return accepted;
  }

  /** Returns whether the frames option has been selected. */
  protected boolean isFramesSelected() {
    return (cbFrames.isEnabled() && cbFrames.isSelected());
  }

  /** Returns whether the center position option has been selected. */
  protected boolean isCenterSelected() {
    return (cbCenter.isEnabled() && cbCenter.isSelected());
  }

  /** Returns whether the cycle definition option has been selected. */
  protected boolean isCyclesSelected() {
    return (cbCycles.isEnabled() && cbCycles.isSelected());
  }

  /** Returns whether the filter configuration option has been selected. */
  protected boolean isFiltersSelected() {
    return (cbFilters.isEnabled() && cbFilters.isSelected());
  }

  /** Returns the version number of the imported configuration file. */
  private int getVersion() {
    return version;
  }

  /** Disposes the dialog and marks it as accepted. */
  private void acceptDialog() {
    setVisible(false);
    accepted = true;
  }

  /** Disposes the dialog and marks it as cancelled. */
  private void cancel() {
    setVisible(false);
    accepted = false;
  }

  /** Initializes the basic dialog layout. */
  private void init() {
    setLayout(new BorderLayout());
    GridBagConstraints c = new GridBagConstraints();

    bAccept.addActionListener(this);
    bCancel.addActionListener(this);

    getRootPane().getInputMap(JComponent.WHEN_IN_FOCUSED_WINDOW).put(KeyStroke.getKeyStroke(KeyEvent.VK_ESCAPE, 0),
        bCancel);
    getRootPane().getActionMap().put(bCancel, new AbstractAction() {
      @Override
      public void actionPerformed(ActionEvent e) {
        cancel();
      }
    });
    getRootPane().getInputMap(JComponent.WHEN_IN_FOCUSED_WINDOW).put(KeyStroke.getKeyStroke(KeyEvent.VK_ENTER, 0),
        bAccept);
    getRootPane().getActionMap().put(bAccept, new AbstractAction() {
      @Override
      public void actionPerformed(ActionEvent e) {
        acceptDialog();
      }
    });

    JPanel pList = new JPanel(new GridBagLayout());
    lSelect.setText(QUESTION_EXPORT);
    c = ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 0, 0, 0), 0, 0);
    pList.add(lSelect, c);
    c = ViewerUtil.setGBC(c, 0, 1, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    pList.add(cbFrames, c);
    c = ViewerUtil.setGBC(c, 0, 2, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 0, 0, 0), 0, 0);
    pList.add(cbCenter, c);
    c = ViewerUtil.setGBC(c, 0, 3, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 0, 0, 0), 0, 0);
    pList.add(cbCycles, c);
    c = ViewerUtil.setGBC(c, 0, 4, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 0, 0, 0), 0, 0);
    pList.add(cbFilters, c);

    JPanel pBottom = new JPanel(new GridBagLayout());
    c = ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    pBottom.add(new JPanel(), c);
    c = ViewerUtil.setGBC(c, 1, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    pBottom.add(bAccept, c);
    c = ViewerUtil.setGBC(c, 2, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 8, 0, 0), 0, 0);
    pBottom.add(bCancel, c);
    c = ViewerUtil.setGBC(c, 3, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    pBottom.add(new JPanel(), c);

    JPanel pMain = new JPanel(new GridBagLayout());
    c = ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.BOTH,
        new Insets(8, 8, 16, 8), 0, 0);
    pMain.add(pList, c);
    c = ViewerUtil.setGBC(c, 0, 1, 1, 1, 0.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.BOTH,
        new Insets(8, 8, 8, 8), 0, 0);
    pMain.add(pBottom, c);

    add(pMain, BorderLayout.CENTER);
    pack();
    setMinimumSize(getPreferredSize());
    setDefaultCloseOperation(WindowConstants.HIDE_ON_CLOSE);
  }
}
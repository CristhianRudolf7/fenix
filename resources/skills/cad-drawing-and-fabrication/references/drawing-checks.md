# Drawing inspection / Inspección de planos

Run `python3 <skill-directory>/scripts/inspect_drawing.py drawing.dxf` in an environment with ezdxf. The helper reads model space, declared units, bounding-box estimates, entity types, layers and open lightweight polylines. Exit 0 means inspection produced a report, **not** manufacturing acceptance. Unknown units yield no millimeter bounds; unsupported entities and unimplemented topology checks remain visible.

Use ezdxf's actual entity and geometry APIs for further checks. For straight/arc paths, assemble contours with an explicit endpoint tolerance and test closure and self-intersections. A list of individually open LINE entities can form a closed contour; counting open polylines alone does not settle connectivity. SPLINE/ELLIPSE/BLOCK geometry needs consumer-specific handling. Keep intended operation layers distinct and compare the bounds and hole dimensions with the design request.

For supplier-specific work, record the current official source, material, thickness, operation and measured feature for each comparison. Network failures and missing catalog fields are unknown, not a failed material or a passed requirement. Keep upload candidates separate from confirmed orders.

ES: el informe básico mide y enumera; no certifica aptitud. Revisa conectividad, solapamientos y restricciones concretas del proveedor con herramientas geométricas antes de declarar una pieza lista para fabricar.

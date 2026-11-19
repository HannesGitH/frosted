// ignore_for_file: public_member_api_docs

part 'in3.primary.copy.gen.dart';

// +mk:copyWith
class const Point(final int x, final int? y);

// +mk:copyWithMixin
class Named({
  required final String? title,
  @Deprecated('x') required final Map<String, List<int>> mapping, // a comment
  final List<String> tags = const ['a, b', "}"],
}) with NamedCopyWith {
  int get count => tags.length;
}

// +mk:copyWith
class OldStyle {
  final String? avatar;
  final String tagType;

  const OldStyle({required this.avatar, required this.tagType});
}

// +mk:copyWithNullableValue
final class const Gen<T extends Object>._(final T value, [var List<T>? others]) implements Comparable<Gen<T>> {
  this : assert(value != null);
  int compareTo(Gen<T> o) => 0;
}

// +mk:copyWith
class WithThis(this.late, {required int super.id}) extends Base {
  late final Set<String>? late;
}
